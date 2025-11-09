// use std::{collections::BTreeMap, io::Write, path::PathBuf, vec};
use serde_json::Value;
// use rust_xlsxwriter::*;

use crate::{to_f64, to_i64, to_str};

use super::to_object;


// pub async fn write_excel_file(path: PathBuf, data: Vec<Value>, ref_header: BTreeMap<String, String>, sort_head: Vec<String>) -> anyhow::Result<()> {
//     let path_file = path.as_os_str().to_str().unwrap();
//     let zip_file = std::fs::File::create(path_file).unwrap();


//     let mut sort_field = sort_head;
//     sort_field.retain(|x| x.to_string() != "*".to_string() && x.to_string() != "".to_string());

//     let mut zip = zip::ZipWriter::new(zip_file);
    
//     let mut chunks_data = data.chunks(500000)
//         .collect::<Vec<_>>();
//     for (part_file, datas) in chunks_data.iter().enumerate() {
//         let part = part_file as i32;
//         let part = part + 1;

//         zip.start_file(format!("{}_{}.xlsx", "export_history_incident", part),  Default::default()).unwrap();  

//         let mut workbook = Workbook::new();
//         let sheet = workbook.add_worksheet();
        
//         for (row_idx, val) in datas.to_vec().into_iter().enumerate() {
//            let data_map = to_object(val);
        
//             if !sort_field.is_empty() {
//                 if row_idx == 0 {
//                     for (col_idx, head) in sort_field.clone().into_iter().enumerate() {
//                         let display_header = ref_header.get(&head).unwrap_or(&format!("{head}")).to_owned();
                        
//                         sheet.write(row_idx as u32, col_idx as u16, display_header).unwrap();
//                     }
//                 }

//                 for (col_idx, head) in sort_field.clone().into_iter().enumerate() {
//                     let value = data_map.get(&head).unwrap_or(&Value::Null).to_owned();

//                     if value.is_f64() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_f64(value)).unwrap();
//                     } else if value.is_i64() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_i64(value)).unwrap();
//                     } else if value.is_string() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_str(value)).unwrap();
//                     } else {
//                         let format1 = Format::new();
                        
//                         sheet.write_blank((row_idx + 1) as u32, col_idx as u16, &format1).unwrap();
//                     }
//                 }
//             } else {
//                 if row_idx == 0 {
//                     let keys = data_map.keys().cloned().into_iter().collect::<Vec<String>>();
    
//                     for (col_idx, head) in keys.clone().into_iter().enumerate() {
//                         let display_header = ref_header.get(&head).unwrap_or(&format!("{head}")).to_owned();
    
//                         sheet.write(row_idx as u32, col_idx as u16, display_header).unwrap();
//                     };  
//                 } 
                
//                 for (col_idx, (_key, value)) in data_map.into_iter().enumerate() {
    
//                     if value.is_f64() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_f64(value)).unwrap();
//                     } else if value.is_i64() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_i64(value)).unwrap();
//                     } else if value.is_string() {
//                         sheet.write((row_idx + 1) as u32, col_idx as u16, to_str(value)).unwrap();
//                     } else {
//                         let format1 = Format::new();
                        
//                         sheet.write_blank((row_idx + 1) as u32, col_idx as u16, &format1).unwrap();
//                     }
//                 }
//             }
//         }

//         let mut buffer = workbook.save_to_buffer().unwrap_or(vec![]);
//         zip.write_all(&buffer).unwrap();

//         buffer.clear();
//     }
    
//     zip.finish().unwrap();

//     chunks_data.clear();

//     Ok(())
// }



pub async fn write_csv_file(data: Vec<Value>, sort_head: Vec<String>, name: String) -> anyhow::Result<(String, Vec<String>)> {
    // let path_file = path.as_os_str().to_str().unwrap();
    // let zip_file = std::fs::File::create(path_file).unwrap();

    let mut sort_field = sort_head;
    sort_field.retain(|x| x.to_string() != "*".to_string() && x.to_string() != "".to_string());


    let file_name = format!("{}.csv", name);
    let mut last_return = Vec::new();
    // let mut zip = zip::ZipWriter::new(zip_file);

    // let mut chunks_data = data.chunks(500000)
    //     .collect::<Vec<_>>();
    // for (part_file, datas) in chunks_data.iter().enumerate() {
        // let part = part_file as i32;
        // let part = part + 1;

        // zip.start_file(format!("{}_{}.csv", name, part),  Default::default()).unwrap();  

        let mut header = Vec::new();
        for (row_idx, val) in data.into_iter().enumerate() {
            let data_map = to_object(val.clone());

            if !sort_field.is_empty() {
                if row_idx == 0 {
                    header = sort_field.clone();

                    // zip.write_all(format!("{}\n", header.join("|")).as_bytes()).unwrap();
                    last_return.push(format!("{}", header.join("|")));
                }

                let mut values = Vec::new();
                for keys in header.clone() {
                    let val = data_map.get(&keys.to_lowercase()).unwrap_or(&Value::Null).to_owned();
    
                    values.push(if val.is_f64() {
                        to_f64(val).to_string()
                    } else if val.is_i64() {
                        to_i64(val).to_string()
                    } else if val.is_string() {
                        to_str(val).replace("\n", "\\n")
                    } else {
                        val.to_string().replace("\n", "\\n")
                    })
                };

                // zip.write_all(format!("{}\n", values.join("|")).as_bytes()).unwrap();
                last_return.push(format!("{}", values.join("|")));
                values.clear();

            } else {
                if row_idx == 0 {
                    header = data_map.keys().cloned().into_iter().collect::<Vec<String>>();

                    // zip.write_all(format!("{}\n", header.join("|")).as_bytes()).unwrap();
                    last_return.push(format!("{}", header.join("|")));
                }

                let mut values = Vec::new();
                for keys in header.clone() {
                    let val = data_map.get(&keys.to_lowercase()).unwrap_or(&Value::Null).to_owned();
    
                    values.push(if val.is_f64() {
                        to_f64(val).to_string()
                    } else if val.is_i64() {
                        to_i64(val).to_string()
                    } else if val.is_string() {
                        to_str(val).replace("\n", "\\n")
                    } else {
                        val.to_string().replace("\n", "\\n")
                    })
                };

                // zip.write_all(format!("{}\n", values.join("|")).as_bytes()).unwrap();
                last_return.push(format!("{}", values.join("|")));
                values.clear();
            }
        }

        header.clear();
    // }

    // chunks_data.clear();

    Ok((file_name, last_return))
}