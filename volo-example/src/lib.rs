#![feature(impl_trait_in_assoc_type)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::RwLock;
use anyhow::Error;
use anyhow::Ok;
use std::time::{SystemTime, UNIX_EPOCH};


pub struct DB{
	kvs: HashMap<String, String>,
}
pub struct Tm{
	kts: HashMap<String, (u128,u128)>,
}
pub struct S{
	contents:RwLock<RefCell<DB>>,
	times:RwLock<RefCell<Tm>>,
}

impl S{
	pub fn new()->Self{
		S{
			contents:RwLock::new(RefCell::new(DB{kvs:HashMap::new()})),
			times:RwLock::new(RefCell::new(Tm{kts:HashMap::new()})),
		}
	}
	pub fn check(&self){
    	let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
		let before = self.times.write().unwrap().borrow().kts.clone();
    	self.times.write().unwrap().borrow_mut().kts.retain(|_, timestamp| now - timestamp.1 <= timestamp.0);
        self.contents.write().unwrap().borrow_mut().kvs.retain(|key, _| (self.times.write().unwrap().borrow().kts.contains_key(key)||(!before.contains_key(key))));
	}
}

unsafe impl Send for S {}
unsafe impl Sync for S {}



#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
    // 这部分是我们需要增加的代码

    async fn get_item(
        &self,
        _req: volo_gen::volo::example::GetItemRequest,
    ) -> core::result::Result<volo_gen::volo::example::GetItemResponse, volo_thrift::AnyhowError>
    {
		
		let mut resp = volo_gen::volo::example::GetItemResponse { op:" ".into(), key:" ".into(), value:" ".into(), state:false};
		let opstr = &(_req.op[..]);
		let key = (&_req.key[..]).to_string();
		let value = (&_req.value[..]).to_string();
		let life:u128 = _req.life.try_into().unwrap();
		self.check();
		//println!("{}",life);
		match opstr {
			"get" => {
				if self.contents.read().unwrap().borrow().kvs.contains_key(&key){
					resp.op = "get".into();
					resp.key = key.clone().into();
					resp.value = self.contents.read().unwrap().borrow().kvs[&key].clone().into();
					resp.state = true;
					return Ok(resp);
				}
				else{
					//println!("hhh");
					resp.op = "get".into();
					resp.key = key.clone().into();
					resp.state = false;
					return Ok(resp);
				}

			}
			"set" => {
				resp.op = "set".into();
				resp.key = key.clone().into();
				if self.contents.read().unwrap().borrow().kvs.contains_key(&key){
					resp.value = self.contents.read().unwrap().borrow().kvs[&key].clone().into();
					resp.state = false;
					return Ok(resp);
				}
				else{
					self.contents.write().unwrap().borrow_mut().kvs.insert(key, value);
					resp.state = true;
					return Ok(resp);
				}
			}
			"setex" => {
				resp.op = "setex".into();
				resp.key = key.clone().into();
				if self.contents.read().unwrap().borrow().kvs.contains_key(&key){
					resp.value = self.contents.read().unwrap().borrow().kvs[&key].clone().into();
					resp.state = false;
					return Ok(resp);
				}
				else{
					self.contents.write().unwrap().borrow_mut().kvs.insert(key.clone(), value);
					self.times.write().unwrap().borrow_mut().kts.insert(key, (life*1000, 
						SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap()
						.as_millis()
					));
					resp.state = true;
					return Ok(resp);
				}
			}
			"del" => {
				resp.op = "del".into();
				resp.key = key.clone().into();
				if self.contents.read().unwrap().borrow().kvs.contains_key(&key){
					self.contents.read().unwrap().borrow_mut().kvs.remove(&key);
					resp.state = true;
					return Ok(resp);
				}
				else{
					resp.state = false;
					return Ok(resp);
				}
			}
			"ping" => {
				//println!("hhh");
				resp.op = "ping".into();
				resp.key = key.clone().into();
				resp.state = true;
				return Ok(resp);
			}
			_ => {
				tracing::error!("Invalid operation!");
			}
		}
        Ok(Default::default())
    }

}

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug + From<Error>,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
		let command = format!("{:?}", &req);
		if command.contains("114514") {
			return Err(S::Error::from(Error::msg("There are inappropriate words and they have been filtered")));
		}
        let resp = self.0.call(cx, req).await;
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}


