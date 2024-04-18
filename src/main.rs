use std::io::Write;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config{
    ip: String,
    domain: String,
    timeout: u64
}

#[tokio::main]
async fn main() {
    let config_file = std::fs::read_to_string("config.json").expect("Failed to open config file");
    let config:Config = serde_json::from_str(&config_file).expect("Invalid config format");

    let pt = "http://";
    let mut file = std::fs::File::create(format!("res({}0).txt", config.ip)).unwrap();
    let res:Vec<(u128, String)> = Vec::new();

    let atomic_res = Arc::new(Mutex::new(res));

    let mut threads = Vec::new();
    let mut ranger = 0u16;
    for _ in 0..4{
        let atomic_res_thread = atomic_res.clone();
        let iprange = config.ip.clone();
        let host = config.domain.clone();
        let th = tokio::task::spawn(async move{
            for i in (ranger)..(ranger+64){
                let ip = format!("{iprange}{i}");
                let mut http = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(format!("{pt}{}", &ip).as_str()).unwrap());
                {
                    let timeout = http.timeout_mut();
                    *timeout = Option::Some(std::time::Duration::from_secs(config.timeout))
                }
                http.headers_mut()
                .append("Host", reqwest::header::HeaderValue::from_str(&host).unwrap());
                let cli = reqwest::Client::new();
    
                let timer = Instant::now();
                let res = cli.execute(http).await;
                let endtime = timer.elapsed();
                
                if res.is_ok() && res.unwrap().status().as_u16()==200{
                    println!("{} {}", &ip, &endtime.as_millis());
                    let mut temp = atomic_res_thread.lock().unwrap();
                    temp.push((endtime.as_millis(), ip))
                }else {
                    println!("Timeout {}", &ip);
                }
            }
        });
        threads.push(th);
        ranger += 64
    }

    for th in threads{
        th.await.unwrap();
    }

    let mut report = atomic_res.lock().unwrap();
    report.sort();
    let mut temp = String::new();
    for both in report.iter(){
        temp = format!("{temp}\n{} {}", both.1, both.0)
    }

    file.write_all(temp.as_bytes()).unwrap()

}
