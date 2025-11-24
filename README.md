# egui_sgr

## Overview

egui_sgr is a Rust library that converts ASCII/ANSI escape sequence color models into colored text in egui. It provides a complete parsing system that can handle all three ANSI color models and intelligently manages foreground and background color states.

## Architectural Design

### Core Components

1.  **AnsiParser** (src/lib.rs)
    *   **Responsibility**: Parses ANSI escape sequences and manages color states.
    *   **State Management**: Uses `current_fg` and `current_bg` to cache the current colors.
    *   **Regular Expression**: `\x1b\[([0-9;]+)m([^\x1b]*)` matches ANSI sequences.
    *   **Key Methods**:
        *   `parse(&mut self, input: &str) -> Vec<ColoredText>`
        *   `process_ansi_sequence(&mut self, sequence: &str)`

2.  **ColoredText Struct**
    *   Represents a text segment with color information.
    *   **Fields**: `text` (String), `foreground_color` (Option<Color32>), `background_color` (Option<Color32>).

3.  **Color Model Modules** (src/color_models/)
    *   Each module is responsible for the conversion logic of one color model.
    *   Provides a unified interface function: `parse_*_color`.

### Color Model Implementation

#### 4-bit Color Model (four_bit.rs)
*   **Handling Range**: 30-37, 40-47, 90-97, 100-107.
*   **Color Mapping**: A predefined array of 16 colors.
*   **Core Function**: `ansi_color_to_egui(color_code: u8) -> Color32`.
*   **Formula**: Converts ANSI codes to an index from 0 to 15.

#### 8-bit Color Model (eight_bit.rs)
*   **Handling Range**: 0-255.
*   **Segments**:
    *   0-15: Standard 16 colors.
    *   16-231: 6×6×6 RGB cube.
    *   232-255: 24 levels of grayscale.
*   **RGB Cube Calculation Formula**:
    ```
    code = color_code - 16
    r = code / 36
    g = (code % 36) / 6
    b = code % 6
    color_component = if c == 0 { 0 } else { 55 + c * 40 } 
    // Simplified, original has different values for c == 5
    ```
*   **Core Function**: `ansi_256_to_egui(color_code: u8) -> Color32`.

#### 24-bit True Color Model (twenty_four_bit.rs)
*   **Handling Format**: `38;2;<r>;<g>;<b>` (foreground) and `48;2;<r>;<g>;<b>` (background).
*   **RGB Range**: 0-255.
*   **Core Function**: `rgb_to_egui(r: u8, g: u8, b: u8) -> Color32`.

## API Design

### Public Interface

1.  **Convenience Function**
    ```rust
    pub fn ansi_to_rich_text(input: &str) -> Vec<RichText>
    ```

2.  **Advanced Interface**
    ```rust
    pub fn convert_to_rich_text(colored_texts: &[ColoredText]) -> Vec<RichText>
    ```

3.  **Parser Interface**
    ```rust
    let mut parser = AnsiParser::new();
    let colored_segments = parser.parse(input);
    ```

### Usage Patterns

1.  **Basic Usage**
    *   Single color: `\x1b[31mRed\x1b[0m`
    *   256 colors: `\x1b[38;5;208mOrange\x1b[0m`
    *   True color: `\x1b[38;2;255;105;180mPink\x1b[0m`

2.  **Combined Usage**
    *   Foreground + Background: `\x1b[31;43mRed text on yellow background\x1b[0m`
    *   Sequence changes: `\x1b[31mRed\x1b[43mRed text on yellow\x1b[32mGreen text on yellow\x1b[0m`

## Color State Management

### State Transition Rules

1.  **Reset**: `\x1b[0m` resets all color states.
2.  **Foreground**: `\x1b[38;...m` sets the foreground color.
3.  **Background**: `\x1b[48;...m` sets the background color.
4.  **Reset Foreground**: `\x1b[39m` resets the foreground color.
5.  **Reset Background**: `\x1b[49m` resets the background color.

### State Caching Algorithm

```
current_state = {fg: None, bg: None}
For each ANSI sequence:
  1. Parse the codes in the sequence.
  2. Update the current state.
  3. Apply the current state to the subsequent text.
```

## Testing Strategy

### Unit Tests

1.  **Color Model Tests**
    *   Verify the correctness of color conversion formulas.
    *   Boundary value testing.
    *   Special value testing.

2.  **Parser Tests**
    *   Parsing of single color sequences.
    *   Parsing of composite color sequences.
    *   State caching tests.
    *   Reset sequence tests.

### Test Coverage

*   4-bit color: 4 tests
*   8-bit color: 2 tests
*   24-bit color: 2 tests
*   Integration tests: 5 tests
*   **Total**: 13 tests

## Performance Considerations

1.  **Regex Pre-compilation**: Avoids repeated compilation overhead.
2.  **State Caching**: Reduces the number of color lookups.
3.  **Memory Efficiency**: Uses `Option` to avoid unnecessary color allocations.

## Dependencies

*   `egui`: Provides `Color32` and `RichText` types.
*   `regex`: Provides regular expression matching functionality.

## Example Application

`examples/demo.rs` demonstrates the full functionality of the library, including:
*   Independent usage of the three color models.
*   Color combination and sequence processing.
*   Interactive color demonstration.