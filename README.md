# JSON to CSV Converter

I've built a application in Rust that converts JSON files to CSV format. 

## Features

### Core Functionality
- I've implemented JSON to CSV conversion with support for both single-object and array-of-objects structures
- You'll see real-time conversion progress tracking
- I've included a preview functionality so you can verify your data before saving

### User Interface
- I've created a modern and intuitive GUI using the egui framework
- You can switch between dark and light themes
- I've added a progress bar with percentage display
- You'll receive clear status updates and error messages
- I've implemented a recent files management system

### CSV Export Options
- I've made the delimiter customizable (comma, semicolon, tab)
- You can choose whether to include headers
- I've added field quoting options
- You can select and reorder columns
- I've made the preview size configurable

### Data Management
- I've built an intuitive column selection interface
- You can search through the preview data
- I've added a recent files list (up to 5 files)
- I've implemented data validation
- You'll get comprehensive error handling and reporting

## Requirements

To run my application, you'll need:
- Rust 1.70 or higher
- Cargo (Rust's package manager)

## Installation

1. Clone my repository:
```bash
git clone <repository-url>
cd json_to_csv_converter
```

2. Build the project:
```bash
cargo build --release
```

## Usage

1. Run my application:
```bash
cargo run --release
```

2. Using the Application:
   - Click "Select JSON File" to choose your input JSON file
   - Use the settings panel (⚙️) to configure export options
   - Click "Convert to CSV" to perform the conversion
   - Preview the data using the "Show Preview" option
   - Click "Save CSV File" to save the converted CSV file

### Settings Panel
- **Theme**: Toggle between dark and light mode
- **CSV Settings**:
  - Delimiter selection (comma, semicolon, tab)
  - Header inclusion toggle
  - Field quoting options
  - Maximum preview rows
- **Column Selection**: Choose which columns to include in the export

### Preview Features
- I've implemented a grid view of the CSV data
- You can search through the data
- I've made the number of preview rows configurable
- I've added striped rows for better readability

## Supported JSON Formats

1. Array of Objects:
```json
[
    {"name": "John", "age": 30},
    {"name": "Jane", "age": 25}
]
```

2. Single Object:
```json
{
    "name": "John",
    "age": 30
}
```

## Error Handling

I've implemented clear error messages for:
- Invalid JSON format
- File reading/writing errors
- Unsupported JSON structures
- Conversion errors
- Data validation issues

## Recent Files

- I've added a system to maintain a list of recently opened files
- You can quickly access previous files from the bottom panel
- I've limited the storage to 5 recent files

## Contributing

I welcome your contributions! Please feel free to submit a Pull Request.

## License

MIT License

## Dependencies

I've used these key dependencies:
- eframe: GUI framework
- serde: JSON parsing
- csv: CSV generation
- rfd: File dialogs
- anyhow: Error handling 
