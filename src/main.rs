use std::{
    fs::File,
    sync::Mutex,
    error::Error,
    net::{TcpListener, TcpStream},
    io::{BufRead, BufReader, BufWriter, Write},
};

static mut THREADS: Vec<std::thread::JoinHandle<()>> = Vec::new(); // Static Thread Vector
lazy_static::lazy_static! {
    /*
        * Create Static Mutex Protected File
        * Used To Log Details Of Each Request
        * Logs In Format:
        * [Request DateTime] [File] [File Size] [CPU End Time In Seconds To 4 Decimal Place] [CPU Time Taken In Seconds To 4 Decimal Place]
    */
    static ref FILE: Mutex<File> = Mutex::new(
        std::fs::OpenOptions::new()
            .create(true) // Create File If It Doesn't Exist
            .append(true) // Open File In Append Mode (Don't Overwrite)
            .open("server_thread.log").unwrap() // Open File
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    /*
        * Multi Threaded Web Server
        * Creates A New Thread For Each Request
    */

    ctrlc::set_handler(move || {
        /*
            * SIGINT Handler
            * Called When Ctrl + C Is Pressed
            * Waits For All Threads To Finish
            * Exits Program
        */
        println!("SIGINT received, shutting down...");
        while unsafe { THREADS.len() } > 0 {
            unsafe {
                THREADS.pop().unwrap().join().unwrap();
            }
        }
        std::process::exit(0);
    })?;

    // Create Listener
    let listener = TcpListener::bind("127.0.0.1:80")?;

    println!("Server running on port 80");

    // Wait For Connections
    for stream in listener.incoming() {
        // Create New Thread For Each Connection
        unsafe {
            THREADS.push(std::thread::spawn(move || {
                // Handle Request
                handle_request(stream.unwrap()).unwrap();
            }));
        }
    }
    Ok(())
}

fn handle_request(p0: TcpStream) -> Result<(), Box<dyn Error>> {
    /*
        * Get CPU Time At Start
        * Handle Request
        * Read Request From Stream
        * Send File To Client
        * Get CPU Time At End
        * Calculate Time Taken
        * Log To File
    */

    let start = std::time::SystemTime::now() // CPU Start Time In Seconds
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_secs_f64();

    let mut stream = BufReader::new(p0); // Create Buffered Reader For Stream
    let mut request = String::new(); // Create Empty String To Store Request

    // Read Request From Stream
    stream.read_line(&mut request)?;

    // Get File Path From Request
    let file = request.split_whitespace().nth(1).unwrap();

    // File Not Found.. Send 404 and Exit
    if !std::path::Path::new(&file[1..]).exists() {
        let mut stream = BufWriter::new(stream.into_inner());
        stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
        return Ok(());
    }
    
    // Create Path to Get File Name
    let path = std::path::Path::new(&file[1..]);
    let file_name = path.file_name().unwrap().to_str().unwrap();

    // Create File Reader
    let mut file = BufReader::new(File::open(&file[1..])?);

    // Create File Writer
    let mut stream = BufWriter::new(stream.into_inner());

    // Send File To Client
    std::io::copy(&mut file, &mut stream)?;

    // Get CPU Time in Seconds At End
    let end = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_secs_f64();

    // Calculate Time Taken
    let time = (end - start) / 10000000000.0;

    // Print To Console File Sent
    println!("Requested File: {} Sent", file_name);

    // Log To File
    writeln!(
        FILE.lock().unwrap(),
        "[{}]\t{}\t{}\t{:.4}\t{:.4}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        file_name,
        file.get_ref().metadata()?.len(),
        end / 10000000000.0,
        time
    )?;

    Ok(())
}