use lazy_static::lazy_static;
use std::net::SocketAddr;
use volo_example::LogLayer;
use std::io;
use std::io::Write;

lazy_static! {
    static ref CLIENT: volo_gen::volo::example::ItemServiceClient = {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        volo_gen::volo::example::ItemServiceClientBuilder::new("volo-example")
            .layer_outer(LogLayer)
            .address(addr)
            .build()
    };
}


#[volo::main]
async fn main() {
    tracing_subscriber::fmt::init();
    loop{
        print!("redis> ");
        let mut input= String::new();
        io::stdout().flush().expect("无法刷新标准输出");
        io::stdin().read_line(&mut input).expect("读取失败！");
        let words: Vec<&str> = input.split_whitespace().collect();
        let input: Vec<String> = words.iter().map(|&s| s.to_string()).collect();
        //println!("单词: {:?}", input);
        let mut req = volo_gen::volo::example::GetItemRequest { op:" ".into(), key:" ".into(), value:" ".into(), life:0i32};
        if words.len() == 0{
            println!("输入为空，请重新输入：");
            continue;
        }
        if input[0].eq("shutdown"){
            break;
        }
        else if input[0].eq("get"){
            if input.len() != 2{
                println!("用法： get <key>, 请重新输入");
                continue;
            }
            req.op = "get".into();
            req.key =  input[1].clone().into();
        }
        else if input[0].eq("set"){
            if input.len() == 3{
                req.op = "set".into();
                req.key = input[1].clone().into();
                req.value =  input[2].clone().into();
            }
            else if input.len() == 5 && input[3].eq("ex"){
                req.op = "setex".into();
                req.key = input[1].clone().into();
                req.value = input[2].clone().into();
                req.life = input[4].clone().trim().parse().expect("您输入的不是正整数");
            }
            else{
                println!("用法： set <key> <value> 或者 set <key> <value> ex <life span> 请重新输入");
                continue;
            }
            
        }
        else if input[0].eq("del"){
            if input.len() != 2{
                println!("用法： del <key>, 请重新输入");
                continue;
            }
            req.op = "del".into();
            req.key = input[1].clone().into();
        }
        else if input[0].eq("ping"){
            req.op = "ping".into();
            req.key = input[1..].join(" ").into();
        }
        else{
            println!("该指令不存在, 请重新输入")
        }
        
        let resp = CLIENT.get_item(req).await;
        match resp {
            Ok(info) => {
                let opstr: &str = &info.op[..];
                let key = (&info.key[..]).to_string();
                let value = (&info.value[..]).to_string();
                let state: bool = info.state;
                match opstr{
                    "get" => {
                        if state == true{
                            println!("The value of key \"{}\" is \"{}\"", key, value);
                        }
                        else{
                            println!("The value of key \"{}\" does not exist", key);
                        }
                    }
                    "set" => {
                        if state == false{
                            println!("The value of key \"{}\" is \"{}\", which already exists", key, value);
                        }
                        else{
                            println!("Successfully inserted");
                        }
                    }
                    "setex" => {
                        if state == false{
                            println!("The value of key \"{}\" is \"{}\", which already exists", key, value);
                        }
                        else{
                            println!("Successfully inserted");
                        }
                    }
                    "del" => {
                        if state == true{
                            println!("Successfully deleted");
                        }
                        else{
                            println!("The value of key \"{}\" does not exist", key);
                        }
                    }
                    "ping" => {
                        if key.eq(""){
                            println!("pong");
                        }
                        else{
                            println!("pong \"{}\"", key);
                        }
                        
                    }
                    

                    _ => tracing::error!("Error!"),
                }
            }
            Err(e) => tracing::error!("{:?}", e),
        }
    }


    //let req = volo_gen::volo::example::GetItemRequest { op:" ".into(), key:" ".into(), value:" ".into() };
    
}

