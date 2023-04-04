use std::{fs};

use regex::Regex as re;
//use arboard::Clipboard;

fn normalize_string(data: &str)->String{
    let normalize_string_r=re::new(r"([0-9/\.,:]\r?\n[0-9/\.,:])").unwrap();
    let mut out_data=data.to_string();
    for m in normalize_string_r.find_iter(out_data.clone().as_str()){
//        println!("z={:?}", &out_data[m.start()..m.end()] );
        let rng=(m.start()+1 ) .. (m.end()-1);        
        out_data.replace_range(rng, "^");
    }
    
    out_data=out_data.replace("\r", "").replace("\n", " ").replace("^", "");


    return out_data;
}

fn main() {
    let mut contents = fs::read_to_string("data.txt")
        .expect("Should have been able to read the file");


    let footer_regexps_str=[
        r"\d{1,2} of \d{1,3}\r?\n?", // page x of x
        r".{1,5}:///tmp[/0-9a-zA-Z\-\.]{10,128}\r?\n?",
        r"\d{1,2}\.\d{1,2}\.\d{4}, \d{1,2}:\d{1,2}", //time
        r"Firefox\r?\n?",
    ];

    let page_top=r"Главное управление Федеральной службы судебных приставов по";
    let page_bottom=r"Результат проверки ЭП: Подпись верна";
    let ust_reg=re::new(r"(?im)(УСТАНОВИЛ.?\r?\n)").unwrap();
    let pst_reg=re::new(r"(?im)(ПОСТАНОВИЛ.?\r?\n)").unwrap();


    for footer_regexp_str in footer_regexps_str{
        contents=re::new(footer_regexp_str).unwrap()
            .replace_all(contents.as_str(), "")
            .to_string();
    }

    let mut pages:Vec<String> = Vec::new();
    contents
        .match_indices(page_top)
        .for_each(|(indx,_)|
            {
                let subpage_not_sliced=&contents[indx..];
                if let Some(btn_indx)=subpage_not_sliced.find(page_bottom){
                    let sliced_page=subpage_not_sliced[..btn_indx].to_string();
                    pages.push( sliced_page );
                }
            }
         );

    let mut decrees:Vec<(String,String,String)> = Vec::new();
    
    println!("for page\n");


    for page in &pages{
        let ustm=ust_reg.find(&page);
        let pstm=pst_reg.find(&page);
        if ustm.is_none() || pstm.is_none() {continue;}
        
        let ustm_last=ustm.unwrap().end();
        let ustm_first=ustm.unwrap().end();
        let pstm_first=pstm.unwrap().start();

        if pstm_first<ustm_last {
            println!("u>p error.(u:{} -> p:{})",ustm_last,pstm_first);
        }

        let head=page[..ustm_first].to_string();
        let ustanovil=normalize_string ( &page[ustm_last..pstm_first] );
        let postanovil=normalize_string( &page[pstm.unwrap().end()..] );
        decrees.push((head,ustanovil,postanovil));
    }
         
    println!("decrees count={} pages count={}",decrees.len(),pages.len());
    let r_isp_num=re::new(r"\d{4,}/\d{1,}/\d{3,}\-[А-Яа-я0-9]{2,}").unwrap();

    let mut string_out:Vec<String>=Vec::new();

    let main_r=re::new(r"Перечислить средства в счет погашения долга взыскателю (?P<org>.*?) \(ИП.*?(?P<ip>\d{4,}/\d{1,}/\d{3,}\-[А-Яа-я0-9]{2,}).*?(?P<sum>[0-9\.,]+) руб").unwrap();


    string_out.push("a,b,c,d".to_string());


    for (_head,ustanovil,postanovil) in &decrees{
            let isp_m=r_isp_num.captures(ustanovil).expect("не найден номер постановления");
            if isp_m.len() !=1 {
                println!("невозможно обработать бланк в котором больше 1 номера постановления");
                continue;
            }

            let isp_num_str:String=isp_m.get(0).unwrap().as_str().to_string();
            let ffs=format!("Произвести распределение денежных средств по исполнительному производству {}", isp_num_str ) ;
            if postanovil.find( &ffs ).is_none(){
                println!("невозможно обработать бланк в котором отсутствует пункт о распределении");
                continue;
            }

            for capture in main_r.captures_iter(&postanovil) {
                    
                string_out.push(
                    format!("{},{},{},{}",
                    isp_num_str,
                    capture.name("ip").unwrap().as_str() ,
                    capture.name("sum").unwrap().as_str(),
                    capture.name("org").unwrap().as_str(),
                )
                );
            }
    }
    fs::write("data.csv", string_out.join("\n") ).expect("can't write file");

}
