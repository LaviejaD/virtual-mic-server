/// Devuelve True o False si existe o no un argumento
pub fn exist(txt: String) -> bool {
    let args: Vec<String> = std::env::args().collect();

    return match args.iter().position(|arg| arg == &txt) {
        Some(_e) => true,
        None => false,
    };
}
/// Devuelve el valor del argumento
pub fn get(txt: String) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

   let r =   args.iter().position(|arg| arg == &txt)
 .and_then(|index| args.get(index+1));
match r { 
Some( e) => Some(e.clone()),
None=> None
}
 }
