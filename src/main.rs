use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};
use std::time::Instant;

use tfhe::integer::{RadixCiphertextBig, ServerKey};

use crossterm::style::{Color, SetForegroundColor};
use crossterm::ExecutableCommand;
use std::io::stdout;

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
    let listener = TcpListener::bind("127.0.0.1:8070")?;
    println!("Server is listening");

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("Requester's tcp client initiated connection\n");
        std::thread::spawn(move || handle_client(stream?));
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    // read an int value from the stream
    // buffer for size of each byte array
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_le_bytes(size_buf) as usize;
    //print size
    println!("Encrypted data size for each value: {}", size);

    // buffers for byte arrays
    let mut gambling_percent_data = vec![0u8; size];
    let mut overspending_score_data = vec![0u8; size];
    let mut impulsive_buying_score_data = vec![0u8; size];
    let mut mean_deposited_sum_data = vec![0u8; size];
    let mut mean_reported_income_data = vec![0u8; size];
    let mut no_months_deposited_data = vec![0u8; size];

    // read byte arrays from stream
    let mut server_key_file = std::fs::File::open("server_key.bin")?;
    let server_key: ServerKey = bincode::deserialize_from(&mut server_key_file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    stream.read_exact(&mut gambling_percent_data)?;
    stream.read_exact(&mut overspending_score_data)?;
    stream.read_exact(&mut impulsive_buying_score_data)?;
    stream.read_exact(&mut mean_deposited_sum_data)?;
    stream.read_exact(&mut mean_reported_income_data)?;
    stream.read_exact(&mut no_months_deposited_data)?;


    let gambling_percent: RadixCiphertextBig = bincode::deserialize(&gambling_percent_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let overspending_score: RadixCiphertextBig = bincode::deserialize(&overspending_score_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let impulsive_buying_score: RadixCiphertextBig = bincode::deserialize(&impulsive_buying_score_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mean_deposited_sum: RadixCiphertextBig = bincode::deserialize(&mean_deposited_sum_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let mean_reported_income: RadixCiphertextBig = bincode::deserialize(&mean_reported_income_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let no_months_deposited: RadixCiphertextBig = bincode::deserialize(&no_months_deposited_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let fhe_computation_now = Instant::now();
    println!("Starting FHE computation");
    let max = server_key.max_parallelized(&overspending_score, &impulsive_buying_score);
    println!("- Max operation done");
    let mut result = server_key.add_parallelized(&gambling_percent, &max);
    println!("- Add operation done");
    let condition = server_key.gt_parallelized(&mean_deposited_sum, &mean_reported_income); // 0 - false, 1 - true
    println!("- Greater than operation done");
    let r = server_key.unchecked_scalar_left_shift(&condition, 1); // 1 left shifted by 1 = 2 , 0 left shifted by 1 = 0
    println!("- Scalar left shift operation done");
    let risk_counter = server_key.mul_parallelized(&r, &no_months_deposited);
    println!("- Multiplication operation done");
    result = server_key.sub_parallelized(&result, &risk_counter);
    println!("- Subtract operation done");
    println!("FHE computation done");


    //write result to file
    let mut result_file = std::fs::File::create("result.bin")?;
    bincode::serialize_into(&mut result_file, &result).unwrap();

    //read result from file
//     let mut result_file = std::fs::File::open("result.bin")?;
//     let result: RadixCiphertextBig = bincode::deserialize_from(&mut result_file)
//         .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let fhe_computation_elapsed = fhe_computation_now.elapsed();
    println!("Processing time is: {:?}", fhe_computation_elapsed);

    let bytes_size = bincode::serialized_size(&result)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let bytes_sizes_be = bytes_size.to_be_bytes();
    stream.write_all(&bytes_sizes_be)?; //all the sizes are the same, so we can send only one
    bincode::serialize_into(&mut stream, &result).unwrap();
    println!("Result sent to requester client.");
    println!("DONE!");
    Ok(())
}