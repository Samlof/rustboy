use super::interconnect::Interconnect;
use super::memory_map;
use crate::memory_map::*;
use crate::utils::check_bit;
use enum_primitive_derive::*;
use minifb::Window;
use minifb::{Key, Scale, WindowOptions};
use num_traits::{FromPrimitive, ToPrimitive};

const VIEWPORT_WIDTH: usize = 160;
const VIEWPORT_HEIGHT: usize = 144;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;
// 20x18 tiles

/*
Horiz Sync: 9198 KHz (9420 KHz for SGB)
Vert Sync: 59.73 Hz (61.17 Hz for SGB)

*/

/*
    8x8 pixep per tile based. 20x18 tiles

    40 sprites
    10 per line

    16 bytes per tile
    2 bytes per line in a tile

    Background tile data, 256 tiles

    Background map 32 tiles, 32 tiles
       256x256 pixels
       Viewport into it with scy and scx
       wraps around

      Window on top of it
      wy, wx control where it is
      Draw to right and bottom

  Sprites
  OAM Entry 4 bytes
    Position x 0x__ 1
  Positiony Y 0x__  1
  Tile Number 0x__  1
  Priority _        rest
  Flip x
  Flip y
  Palette

  Sprite has width of 8, height of 16
  Bind point at bottom right
  40 sprites in OAM ram FE00 -> FE9C. Last sprite numbers are drawn behind

  Palette0 transparent, black, light gray, white
  Palette1 transparent, dark gray, light gray, white

  Sprite on left is drawn on top


// 4kb sprite tiles 256 tiles x 16 bytes
// 4kb BG tiles 256 tiles x 16 bytes
// 1kb BG map 32x32 indexes
// 1kb window map 32x32 indexes

0x8000 - 0x9800 -> sprite and map tiles

0x9800-> 0x9900 bg map
0x9900 -> 0xA000 window map
*/

#[derive(Debug, PartialEq, Primitive, Clone, Copy)]
pub enum Color {
    White = 0b00,
    LightGray = 0b01,
    DarkGray = 0b10,
    Black = 0b11,
}

#[derive(Debug, PartialEq)]
enum State {
    OAMSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

#[allow(non_snake_case)]
pub struct Ppu {
    LCD_control: u8, // FF40
    LCDC_status: u8, // FF41
    scy: u8,         // FF42
    scx: u8,         // FF43
    ly: u8,          // FF44
    lyc: u8,         // FF45
    bgp: u8,         // FF47
    obp0: u8,        // FF48
    obp1: u8,        // FF49
    wy: u8,          // FF4A
    wx: u8,          // FF4B

    pub main_window: Window,

    sprite_memory: Box<[u8]>,
    vram: Box<[u8]>,

    buffer: Vec<u8>,
    viewport_buffer: Vec<u32>,

    cycles: i32,
    state: State,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            LCD_control: 0x91,
            LCDC_status: 0,
            ly: 0,
            lyc: 0,
            scy: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            scx: 0,

            sprite_memory: vec![0; SPRITE_MEM_LENGTH as usize].into_boxed_slice(),
            vram: vec![0; VRAM_LENGTH as usize].into_boxed_slice(),

            main_window: create_window(VIEWPORT_WIDTH, VIEWPORT_HEIGHT, "Rustboy", Scale::X4),

            buffer: vec![0; WIDTH * HEIGHT],
            viewport_buffer: vec![0; VIEWPORT_WIDTH * VIEWPORT_HEIGHT],
            cycles: 0,
            state: State::OAMSearch,
        }
    }

    // bool signifies whether a vblank interrupt or not
    pub fn update(&mut self) -> bool {
        // If on cooldown, jump out
        if self.cycles > 0 {
            self.cycles -= 1;
            return false;
        }
        match self.state {
            State::OAMSearch => {
                self.cycles = 20;
                // Change status
                self.state = State::PixelTransfer;
                self.LCDC_status |= 0b11;
            }
            State::PixelTransfer => {
                self.cycles = 43;

                self.pixel_transfer();
                // Change status
                self.state = State::HBlank;
                self.LCDC_status &= !0b11;
            }
            State::HBlank => {
                self.cycles = 51;
                self.ly += 1;
                self.state = if self.ly == 144 {
                    self.LCDC_status &= !0b11;
                    self.LCDC_status |= 0b01;
                    State::VBlank
                } else {
                    self.LCDC_status &= !0b11;
                    self.LCDC_status |= 0b10;
                    State::OAMSearch
                };
            }
            State::VBlank => {
                self.ly += 1;
                self.cycles = 114;

                if self.ly == 154 {
                    self.ly = 0;

                    self.LCDC_status &= !0b11;
                    self.LCDC_status |= 0b10;
                    self.state = State::OAMSearch;
                }
                if self.ly == 145 {
                    self.main_window
                        .update_with_buffer(&*self.viewport_buffer)
                        .unwrap();
                    return true;
                }
            }
        }
        return false;
    }

    pub fn turn_lcd_off(&mut self) {
        self.disable_lcd();
        // TODO: pause ppu and draw black?
    }
    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            0xFF40 => Some(self.LCD_control),
            0xFF41 => Some(self.LCDC_status),
            0xFF42 => Some(self.scy),
            0xFF43 => Some(self.scx),
            0xFF44 => Some(self.ly),
            0xFF45 => Some(self.lyc),
            0xFF47 => Some(self.bgp),
            0xFF48 => Some(self.obp0),
            0xFF49 => Some(self.obp1),
            0xFF4A => Some(self.wy),
            0xFF4B => Some(self.wx),
            _ => None,
        }
    }

    pub fn pixel_transfer(&mut self) {
        if !self.lcd_display_enabled() {
            return;
        }
        self.draw_background();
        self.draw_sprites();
    }

    fn draw_background(&mut self) {
        // scy is the viewport top. ly is which line in the viewport
        let line = self.ly as u16 + self.scy as u16;
        let line = line % VIEWPORT_HEIGHT as u16;
        // Same but for column
        let column = self.scx;

        // Move background pixels
        for i in 0..VIEWPORT_WIDTH {
            let color = self.buffer[(line as usize * WIDTH) + (column as usize + i) % WIDTH];
            self.viewport_buffer[(self.ly as usize * VIEWPORT_WIDTH) + i] =
                bg_bit_into_color(color);
        }
    }

    fn draw_sprites(&mut self) {
        if !self.obj_enable() {
            return;
        }
        let sprite_height = self.obj_height();

        // Loop thru all the sprites
        for sprite in (0..40).map(|x| x * 4) {
            let sprite = create_sprite(&self.sprite_memory, sprite, false);
            // Check if the sprite is on this line
            if self.ly < sprite.y || self.ly >= sprite.y + sprite_height {
                continue;
            }
            // Check if x is visible
            // FIXME:
            if sprite.x == 0 || sprite.x >= 168 {
                //continue;
            }
            // Draw the right line
            // sprite.y - self.ly gives the distance from bottom of the sprite
            // sprite_height - that to give it from top
            let line_to_draw = self.ly - sprite.y;

            if sprite_height == 8 {
                let bytes_to_skip = line_to_draw as u16 * 2;
                let tile_addr = 0x8000 + sprite.tile_nr as u16 * 16;
                let byte1 = self.get_from_vram(tile_addr + bytes_to_skip);
                let byte2 = self.get_from_vram(tile_addr + bytes_to_skip + 1);

                for j in 0..8 {
                    let buffer_col = sprite.x + j;
                    if buffer_col > VIEWPORT_WIDTH as u8 {
                        continue;
                    }
                    let color = ((byte1 >> (7 - j)) & 1) | (((byte2 >> (7 - j)) & 1) << 1);
                    if color == 0 {
                        // color of 0 is transparent for sprites
                        continue;
                    }

                    self.viewport_buffer
                        [(self.ly as usize * VIEWPORT_WIDTH) + buffer_col as usize] =
                        bg_bit_into_color(color);
                }
            }
            // TODO: sprite_height of 16
        }
    }

    fn update_bg_tile(&mut self, map_addr: u16, tile_data_nr: u8) {
        let tile_size = 16; // one tile is 16 bytes
        let tile_data_start = self.bg_window_tile_data();
        let tile_addr = if tile_data_start == 0x8800 {
            // tile index is -128 - 127. 0 at 0x9000
            // Sign extend and change to i16 for address
            let tile_data_nr = tile_data_nr as i8 as i16;
            (0x9000u16 as i16 + (tile_data_nr * tile_size as i16)) as u16
        } else {
            // tile index is 0-255. 0 at 0x8000
            tile_data_start + (tile_data_nr as u16 * tile_size as u16)
        };

        let tile_map_nr = map_addr - self.bg_tile_map_address();
        // 32 tiles per row. so tile_nr/32 gives tile row. Then 8 pixels each tile
        let buffer_start_row_pixel = (tile_map_nr / 32) * 8;
        let buffer_start_column_pixel = (tile_map_nr % 32) * 8;

        self.draw_tile(buffer_start_row_pixel, buffer_start_column_pixel, tile_addr);
    }

    fn draw_tile(
        &mut self,
        buffer_start_row_pixel: u16,
        buffer_start_column_pixel: u16,
        tile_addr: u16,
    ) {
        // Update the 8x8 area
        for i in 0..8 {
            let buffer_row = buffer_start_row_pixel + i;
            let byte1 = self.get_from_vram(tile_addr + i * 2);
            let byte2 = self.get_from_vram(tile_addr + i * 2 + 1);

            for j in 0..8 {
                let buffer_col = buffer_start_column_pixel + j;

                let color = (byte1 >> (7 - j) & 1) | ((byte2 >> (7 - j) & 1) << 1);
                self.buffer[(buffer_row as usize * WIDTH) + buffer_col as usize] = color;
            }
        }
    }

    fn get_from_vram(&self, address: u16) -> u8 {
        let address = address - VRAM_START;
        self.vram[address as usize]
    }

    pub fn read_vram(&self, address: u16) -> u8 {
        if self.state == State::PixelTransfer {
            //return 0xFF;
        }
        let address = address - VRAM_START;
        self.vram[address as usize]
    }
    pub fn write_vram(&mut self, address: u16, value: u8) {
        if self.state == State::PixelTransfer {
            //return;
        }
        let vram_address = address - VRAM_START;
        self.vram[vram_address as usize] = value;

        if self.is_addr_in_bg_map(address) {
            self.update_bg_tile(address, value);
        }
    }

    fn is_addr_in_bg_map(&self, address: u16) -> bool {
        if self.bg_tile_map_address() == 0x9800 {
            address >= 0x9800 && address < 0x9BFF
        } else {
            address >= 0x9C00 && address < 0x9FFF
        }
    }

    pub fn read_sprite_mem(&self, address: u16) -> u8 {
        if self.state == State::PixelTransfer || self.state == State::OAMSearch {
            //return 0xFF;
        }
        let address = address - SPRITE_MEM_START;
        self.sprite_memory[address as usize]
    }
    pub fn write_sprite_mem(&mut self, address: u16, value: u8) {
        if self.state == State::PixelTransfer || self.state == State::OAMSearch {
            //return;
        }
        let address = address - SPRITE_MEM_START;
        self.sprite_memory[address as usize] = value;
    }

    pub fn write(&mut self, address: u16, value: u8) -> bool {
        match address {
            0xFF40 => self.LCD_control = value,
            0xFF41 => self.LCDC_status = value,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {
                // ly is reset on write
                self.ly = 154;
                self.state = State::VBlank;
            }
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,

            _ => return false,
        }
        true
    }

    fn disable_lcd(&mut self) {
        self.LCD_control &= !(1 << 7);
    }

    fn enable_lcd(&mut self) {
        self.LCD_control |= 1 << 7;
    }
    fn lcd_display_enabled(&self) -> bool {
        self.LCD_control & (1 << 7) > 0
    }
    fn window_tile_map_address(&self) -> u16 {
        if self.LCD_control & (1 << 6) > 0 {
            0x9C00
        } else {
            0x9800
        }
    }
    fn window_enable(&self) -> bool {
        self.LCD_control & (1 << 5) > 0
    }
    fn bg_window_tile_data(&self) -> u16 {
        if self.LCD_control & (1 << 4) > 0 {
            0x8000 // Same are as obj
        } else {
            0x8800
        }
    }
    fn bg_tile_map_address(&self) -> u16 {
        if self.LCD_control & (1 << 3) > 0 {
            0x9C00
        } else {
            0x9800
        }
    }
    fn obj_height(&self) -> u8 {
        if self.LCD_control & (1 << 2) > 0 {
            16
        } else {
            8
        }
    }
    fn obj_enable(&self) -> bool {
        self.LCD_control & (1 << 1) > 0
    }
    fn bg_enable(&self) -> bool {
        self.LCD_control & 1 > 0
    }

    fn lyc_ly_interrupt(&self) -> bool {
        self.LCDC_status & (1 << 6) > 0
    }
    fn mode_2_oam_interrupt(&self) -> bool {
        self.LCDC_status & (1 << 5) > 0
    }
    fn mode_1_vblank_interrupt(&self) -> bool {
        self.LCDC_status & (1 << 4) > 0
    }
    fn mode_0_hblank_interrupt(&self) -> bool {
        self.LCDC_status & (1 << 3) > 0
    }
    fn lyc_ly_flag(&self) -> bool {
        self.LCDC_status & (1 << 2) > 0
    }
    fn lcdc_status_mode(&self) -> u8 {
        self.LCDC_status & 0b11
    }

    fn bg_color(&self, value: u8) -> Color {
        match value {
            0 => color_for_00(self.bgp),
            1 => color_for_01(self.bgp),
            2 => color_for_10(self.bgp),
            3 => color_for_11(self.bgp),
            _ => Color::Black,
        }
    }

    pub fn add_cycles(&mut self, c: i32) {
        self.cycles += c;
    }
}

#[derive(Debug)]
struct Sprite {
    y: u8,
    x: u8,
    tile_nr: u8,
    above_bg: bool,
    y_flip: bool,
    x_flip: bool,
    palette_nr: u8,
    tile_vram_bank: u8,
}

fn create_sprite(oam_mem: &[u8], address: usize, cgb_mode: bool) -> Sprite {
    Sprite {
        y: oam_mem[address] - 16,
        x: oam_mem[address + 1] - 8,
        tile_nr: oam_mem[address + 2],
        above_bg: !check_bit(oam_mem[address + 3], 7),
        y_flip: check_bit(oam_mem[address + 3], 6),
        x_flip: check_bit(oam_mem[address + 3], 5),
        palette_nr: oam_mem[address + 3] & if cgb_mode { 0x07 } else { 0x10 },
        tile_vram_bank: oam_mem[address + 3] & 0x08,
    }
}

fn create_window(width: usize, height: usize, title: &str, scale: Scale) -> Window {
    let opts = WindowOptions {
        borderless: false,
        title: true,
        resize: false,
        scale: scale,
    };
    let mut window = Window::new(title, width, height, opts).unwrap_or_else(|e| {
        panic!("{}", e);
    });
    return window;
}

fn bg_bit_into_color(bit: u8) -> u32 {
    match bit {
        0b00 => 0xffffff,
        0b01 => 0x505151,
        0b10 => 0x838484,
        0b11 => 0,
        _ => panic!("Invalid color!"),
    }
}

fn color_for_11(palette: u8) -> Color {
    Color::from_u8((palette >> 6) & 0b11).unwrap()
}
fn color_for_10(palette: u8) -> Color {
    Color::from_u8((palette >> 4) & 0b11).unwrap()
}
fn color_for_01(palette: u8) -> Color {
    Color::from_u8((palette >> 2) & 0b11).unwrap()
}
fn color_for_00(palette: u8) -> Color {
    Color::from_u8(palette & 0b11).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
}

/*
35. FF44 (LY)
 Name - LY
 Contents - LCDC Y-Coordinate (R)
 The LY indicates the vertical line to which
 the present data is transferred to the LCD
 Driver. The LY can take on any value
 between 0 through 153. The values between
 144 and 153 indicate the V-Blank period.
 Writing will reset the counter.

33. FF42 (SCY)
 Name - SCY
 Contents - Scroll Y (R/W)
 8 Bit value $00-$FF to scroll BG Y screen
 position.


31. FF40 (LCDC)
 Name - LCDC (value $91 at reset)
 Contents - LCD Control (R/W)
 Bit 7 - LCD Control Operation *
 0: Stop completely (no picture on screen)
 1: operation
 Bit 6 - Window Tile Map Display Select
 0: $9800-$9BFF
 1: $9C00-$9FFF
 Bit 5 - Window Display
 0: off
 1: on
 Bit 4 - BG & Window Tile Data Select
 0: $8800-$97FF
 1: $8000-$8FFF <- Same area as OBJ
 Bit 3 - BG Tile Map Display Select
 0: $9800-$9BFF
 1: $9C00-$9FFF
 Bit 2 - OBJ (Sprite) Size
 0: 8*8
 1: 8*16 (width*height)
 Bit 1 - OBJ (Sprite) Display
 0: off
 1: on
 Bit 0 - BG & Window Display
 0: off
 1: on
 * - Stopping LCD operation (bit 7 from 1 to 0) must
 be performed during V-blank to work properly. V-
 blank can be confirmed when the value of LY is
 greater than or equal to 144.



38. FF47 (BGP)
 Name - BGP
 Contents - BG & Window Palette Data (R/W)
 Bit 7-6 - Data for Dot Data 11
 (Normally darkest color)
 Bit 5-4 - Data for Dot Data 10
 Bit 3-2 - Data for Dot Data 01
 Bit 1-0 - Data for Dot Data 00
 (Normally lightest color)
 This selects the shade of grays to use
 for the background (BG) & window pixels.
 Since each pixel uses 2 bits, the
 corresponding shade will be selected from
 here.

*/
