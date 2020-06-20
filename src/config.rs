/******************************************************************************/
/*                               libRustConfig                                */
/*                   rust wrapper around libconfig library                    */
/*                  https://github.com/hyperrealm/libconfig                   */
/*                                                                            */
/* Copyright (c) 2020                                       Ivan Semenkov     */
/* https://github.com/isemenkov/librustconfig               ivan@semenkov.pro */
/*                                                          Ukraine           */
/******************************************************************************/
/*                                                                            */
/* Permission is hereby granted,  free of charge,  to any person obtaining a  */
/* copy of this software and associated documentation files (the "Software"), */
/* to deal in the Software without restriction, including without limitation  */
/* the rights to use, copy,  modify, merge, publish, distribute,  sublicense, */
/* and/or  sell copies  of the Software,  and to permit persons  to whom  the */
/* Software  is furnished to  do  so,  subject to  the following  conditions: */
/*                                                                            */
/* The above copyright notice and this permission notice shall be included in */
/* all copies or substantial portions of the Software.                        */
/*                                                                            */
/* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR */
/* IMPLIED,  INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF  MERCHANTABILITY, */
/* FITNESS  FOR A PARTICULAR PURPOSE  AND NONINFRINGEMENT. IN  NO EVENT SHALL */
/* THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER */
/* LIABILITY,  WHETHER IN AN ACTION  OF CONTRACT,  TORT OR OTHERWISE, ARISING */
/* FROM,  OUT OF  OR IN  CONNECTION WITH  THE SOFTWARE  OR THE  USE OR  OTHER */
/* DEALINGS IN THE SOFTWARE.                                                  */
/*                                                                            */
/******************************************************************************/

use libconfig_sys as raw;

use std::{mem::MaybeUninit, path};
use std::ffi::{CStr, CString};

/// Configuration file.
pub struct Config {
    config : raw::config_t,
    root_element : Option<*mut raw::config_setting_t>
}

/// Option value type.
#[derive(Debug, PartialEq)]
pub enum OptionType {
    IntegerType,
    Int64Type,
    FloatType,
    StringType,
    BooleanType
}

/// Writer for configuration option.
#[derive(Clone, Copy)]
pub struct OptionWriter {
    element : Option<*mut raw::config_setting_t>
}

/// Reader for configuration option.
pub struct OptionReader {
    element : Option<*mut raw::config_setting_t>
}

/// Config errors codes.
#[derive(Debug, PartialEq)]
pub enum Errors {
    ParseError,
    FileNotExists,
    SaveError,
    ElementNotExists,
    DeleteError
}

/// Config result type.
type Result<T> = std::result::Result<T, Errors>;

impl Config {
    
    /// Constructor.
    /// Create new Config struct.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// ```
    pub fn new() -> Config {
        let mut c = MaybeUninit::<raw::config_t>::uninit();
        let cfg = unsafe {
            raw::config_init(c.as_mut_ptr());
            c.assume_init()
        };
        
        let option = raw::config_root_setting(&cfg);
        let element = {    
            if option.is_null() {
                None
            } else {
                Some(option)
            }
        };
    
        Config {
            config : cfg,
            root_element : element
        }
    }
    
    /// Load config file from file and parse it.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// use std::path::Path;
    /// 
    /// let mut cfg = Config::new();
    /// if cfg.load_from_file(Path::new("test.cfg")).is_ok() {
    ///     // ...
    /// }
    /// ```
    pub fn load_from_file(&mut self, file_name : &path::Path) -> Result<()> {
        if file_name.exists() {
            unsafe {
                let result = raw::config_read_file(&mut self.config, 
                    CString::new(file_name.as_os_str().to_str().unwrap())
                        .unwrap().as_ptr()
                );
                
                if result == raw::CONFIG_TRUE {
                    self.root_element = 
                        Some(raw::config_root_setting(&self.config));
                    Ok(())
                } else {
                    self.root_element = None;
                    Err(Errors::ParseError)
                }
            }
        } else {
            Err(Errors::FileNotExists)
        }
    }
    
    /// Parse configuration from string.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let mut cfg = Config::new();
    /// if cfg.load_from_string("root { value = 1 }").is_ok() {
    ///     // ...
    /// }
    /// ```
    pub fn load_from_string<S>(&mut self, config_string : S) -> Result<()>
        where S: Into<String> {
          
        let result = unsafe { 
            raw::config_read_string(&mut self.config, 
                CString::new(config_string.into()).unwrap().as_ptr())
        };
        
        if result == raw::CONFIG_TRUE {
            let option = raw::config_root_setting(&self.config);
            
            if option.is_null() {
                self.root_element = None;
                Err(Errors::ParseError)
            } else {
                self.root_element = Some(option);
                Ok(())
            }
        } else {
            self.root_element = None;
            Err(Errors::ParseError)
        }
    }
   
   /// Save current config to file.
   /// 
   /// # Example
   /// ```
   /// use libconfig::config::Config;
   /// use std::path::Path;
   /// use std::fs;
   /// 
   /// let mut cfg = Config::new();
   /// if cfg.save_to_file(Path::new("test.cfg")).is_ok() {
   ///      // ...
   /// }
   /// fs::remove_file(Path::new("test.cfg"));
   /// ```
    pub fn save_to_file(&mut self, file_name : &path::Path) -> Result<()> {
        let result = unsafe { raw::config_write_file(&mut self.config, 
            CString::new(file_name.as_os_str().to_str().unwrap())
                .unwrap().as_ptr())
        };
        
        if result == raw::CONFIG_TRUE {
            Ok(())
        } else {
            Err(Errors::SaveError)
        }
    }
    
    /// Set current config include directory.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// use std::path::Path;
    /// 
    /// let mut cfg = Config::new();
    /// cfg.include_dir(Path::new("/config"));
    /// ```
    pub fn include_dir(&mut self, path : &path::Path) -> () {
        unsafe {
            raw::config_set_include_dir(&mut self.config, 
                CString::new(path.as_os_str().to_str().unwrap())
                    .unwrap().as_ptr())
        }
    }
        
    /// Read value from path.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value") {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn value<S>(&self, path : S) -> Option<OptionReader>
        where S: Into<String> {
        OptionReader::new(self.root_element).value(path)
    }
    
    /// Create new group section.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("root") {
    ///     Some(s) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn create_section<S>(&self, path : S) -> Option<OptionWriter>
        where S: Into<String> {
        OptionWriter::new(self.root_element).create_section(path)
    }
}

/// Destructor.
/// Clear config and delete all allocated memory data.
impl Drop for Config {
    fn drop (&mut self) {
        unsafe { 
            raw::config_destroy(&mut self.config); 
        }
    }
}

impl OptionWriter {
    
    // Constructor.
    fn new(elem : Option<*mut raw::config_setting_t>) -> OptionWriter {
        OptionWriter {
            element : elem
        }
    }
    
    /// Delete current config element.
    pub fn delete(&self) -> Result<()> {
        if self.element.is_none() {
            return Err(Errors::ElementNotExists)
        }

        if OptionReader::new(self.element).is_section().unwrap() {
            let result = {
                let name = raw::config_setting_name(self.element.unwrap());
                
                if name.is_null() {
                    return Err(Errors::DeleteError);
                }

                let parent = OptionReader::new(self.element).parent();
                if parent.is_none() {
                    return Err(Errors::DeleteError);
                }

                unsafe { raw::config_setting_remove(parent.unwrap()
                    .element.unwrap(), name) }
            };

            match result {
                raw::CONFIG_TRUE => { Ok(()) },
                _ => { Err(Errors::DeleteError) }
            }
        } else {
            let result = {
                let parent = OptionReader::new(self.element).parent();
                let index = unsafe { 
                    raw::config_setting_index(self.element.unwrap())
                };

                if parent.is_none() {
                    return Err(Errors::DeleteError);
                }

                unsafe { raw::config_setting_remove_elem(parent.unwrap()
                    .element.unwrap(), index as u32) }
            };

            match result {
                raw::CONFIG_TRUE => { Ok(()) },
                _ => { Err(Errors::DeleteError) }
            }
        }
    }

    /// Create new group section.
    /// 
    /// # Examples
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("root.group") {
    ///     Some(s) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn create_section<S>(&self, path : S) -> Option<OptionWriter> 
        where S: Into<String> {
            
        if self.element.is_none() {
            return None
        }
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(), 
                CString::new(path.into()).unwrap().as_ptr(), 
                raw::CONFIG_TYPE_GROUP as i32)
        };
        
        if option.is_null() {
            None
        } else {
            Some(OptionWriter::new(Some(option)))
        }
    }
    
    /// Create new array group section.
    /// 
    /// # Examples
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_array("root.array") {
    ///     Some(s) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn create_array<S>(&self, path : S) -> Option<OptionWriter> 
        where S: Into<String> {
            
        if self.element.is_none() {
            return None
        }
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(), 
                CString::new(path.into()).unwrap().as_ptr(), 
                raw::CONFIG_TYPE_ARRAY as i32)
        };
        
        if option.is_null() {
            None
        } else {
            Some(OptionWriter::new(Some(option)))
        }
    }

    /// Create new list group section.
    /// 
    /// # Examples
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_list("root.list") {
    ///     Some(s) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn create_list<S>(&self, path : S) -> Option<OptionWriter> 
        where S: Into<String> {
            
        if self.element.is_none() {
            return None
        }
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(), 
                CString::new(path.into()).unwrap().as_ptr(), 
                raw::CONFIG_TYPE_LIST as i32)
        };
        
        if option.is_null() {
            None
        } else {
            Some(OptionWriter::new(Some(option)))
        }
    }

    /// Add new integer value to current group.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("section") {
    ///     Some(s) => { 
    ///         s.write_int32("ival", 321); 
    ///     },
    ///     None => { /* ... */ }
    /// ```
    pub fn write_int32<S>(&self, name : S, value : i32) -> 
        Option<OptionWriter> where S: Into<String> {
            
        if self.element.is_none() {
            return None
        };
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(),
                CString::new(name.into()).unwrap().as_ptr(),
                raw::CONFIG_TYPE_INT as i32)
        };
        
        if option.is_null() {
            None
        } else {
            let result = unsafe {
                raw::config_setting_set_int(option, value)  
            };
            
            if result == raw::CONFIG_TRUE {
                Some(*self)
            } else {
                None
            }
        }
    }

    /// Add new int64 value to current group.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("section") {
    ///     Some(s) => { 
    ///         s.write_int64("ival", 321000); 
    ///     },
    ///     None => { /* ... */ }
    /// ```
    pub fn write_int64<S>(&self, name : S, value : i64) -> 
        Option<OptionWriter> where S: Into<String> {
            
        if self.element.is_none() {
            return None
        };
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(),
                CString::new(name.into()).unwrap().as_ptr(),
                raw::CONFIG_TYPE_INT64 as i32)
        };
        
        if option.is_null() {
            None
        } else {
            let result = unsafe {
                raw::config_setting_set_int64(option, value)  
            };
            
            if result == raw::CONFIG_TRUE {
                Some(*self)
            } else {
                None
            }
        }
    }

    /// Add new float value to current group.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("section") {
    ///     Some(s) => { 
    ///         s.write_float64("ival", 321.001); 
    ///     },
    ///     None => { /* ... */ }
    /// ```
    pub fn write_float64<S>(&self, name : S, value : f64) -> 
        Option<OptionWriter> where S: Into<String> {
            
        if self.element.is_none() {
            return None
        };
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(),
                CString::new(name.into()).unwrap().as_ptr(),
                raw::CONFIG_TYPE_FLOAT as i32)
        };
        
        if option.is_null() {
            None
        } else {
            let result = unsafe {
                raw::config_setting_set_float(option, value)  
            };
            
            if result == raw::CONFIG_TRUE {
                Some(*self)
            } else {
                None
            }
        }
    }

    /// Add new boolean value to current group.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("section") {
    ///     Some(s) => { 
    ///         s.write_bool("ival", false); 
    ///     },
    ///     None => { /* ... */ }
    /// ```
    pub fn write_bool<S>(&self, name : S, value : bool) -> 
        Option<OptionWriter> where S: Into<String> {
            
        if self.element.is_none() {
            return None
        };
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(),
                CString::new(name.into()).unwrap().as_ptr(),
                raw::CONFIG_TYPE_BOOL as i32)
        };
        
        if option.is_null() {
            None
        } else {
            let val = {
                match value {
                    true => { raw::CONFIG_TRUE },
                    false => { raw::CONFIG_FALSE }
                }
            };
            let result = unsafe {
                raw::config_setting_set_bool(option, val)  
            };
            
            if result == raw::CONFIG_TRUE {
                Some(*self)
            } else {
                None
            }
        }
    }

    /// Add new string value to current group.
    /// 
    /// # Example
    /// ```
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.create_section("section") {
    ///     Some(s) => { 
    ///         s.write_string("ival", "test string"); 
    ///     },
    ///     None => { /* ... */ }
    /// ```
    pub fn write_string<S>(&self, name : S, value : S) -> 
        Option<OptionWriter> where S: Into<String> {
            
        if self.element.is_none() {
            return None
        };
        
        let option = unsafe {
            raw::config_setting_add(self.element.unwrap(),
                CString::new(name.into()).unwrap().as_ptr(),
                raw::CONFIG_TYPE_STRING as i32)
        };
        
        if option.is_null() {
            None
        } else {
            let result = unsafe {
                raw::config_setting_set_string(option, 
                    CString::new(value.into()).unwrap().as_ptr())  
            };
            
            if result == raw::CONFIG_TRUE {
                Some(*self)
            } else {
                None
            }
        }
    }
}

impl OptionReader {
    
    // Constructor
    fn new(elem : Option<*mut raw::config_setting_t>) -> OptionReader {
        OptionReader {
            element : elem
        }
    }

    /// Delete current config element.
    pub fn delete(&self) -> Result<()> {
        OptionWriter::new(self.element).delete()
    }
    
    /// Return true if element is section group.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// if cfg.value("root").unwrap().is_section().unwrap() {
    ///     // ...
    /// }
    /// ``` 
    pub fn is_section(&self) -> Option<bool> {
        if self.element.is_none() {
            return None;
        }
        
        let result = raw::config_setting_is_group(self.element.unwrap());
        Some(result == raw::CONFIG_TRUE)      
    }
    
    /// Return true if element is array.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// if cfg.value("root.value").unwrap().is_array().unwrap() {
    ///     // ...
    /// }
    /// ```
    pub fn is_array(&self) -> Option<bool> {
        if self.element.is_none() {
            return None
        }
        
        let result = raw::config_setting_is_array(self.element.unwrap());
        Some(result == raw::CONFIG_TRUE)
    }
    
    /// Return true if element is list.
    ///
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// if cfg.value("root.value").unwrap().is_list().unwrap() {
    ///     // ...
    /// }
    /// ```
    pub fn is_list(&self) -> Option<bool> {
        if self.element.is_none() {
            return None
        }
        
        let result = raw::config_setting_is_list(self.element.unwrap());
        Some(result == raw::CONFIG_TRUE)
    }
    
    /// Return option element parent item.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.section").unwrap().parent() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn parent(&self) -> Option<OptionReader> {
        if self.element.is_none() {
            return None
        }
        
        let result = raw::config_setting_parent(self.element.unwrap());
        
        if result.is_null() {
            None
        } else {
            Some(OptionReader::new(Some(result)))
        }
    }
    
    /// Return option value type.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::{Config, OptionType};
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().value_type().unwrap() {
    ///     OptionType::IntegerType => { /* ... */ },
    ///     OptionType::Int64Type => { /* ... */ },
    ///     OptionType::FloatType => { /* ... */ },
    ///     OptionType::StringType => { /* ... */ },
    ///     OptionType::BooleanType => { /* ... */ }
    /// }
    /// ```
    pub fn value_type(&self) -> Option<OptionType> {
        if self.element.is_none() {
            return None
        }
        
        let result = raw::config_setting_type(self.element.unwrap());
        match result as i16 {
            raw::CONFIG_TYPE_INT => { Some(OptionType::IntegerType) },
            raw::CONFIG_TYPE_INT64 => { Some(OptionType::Int64Type) },
            raw::CONFIG_TYPE_FLOAT => { Some(OptionType::FloatType) },
            raw::CONFIG_TYPE_STRING => { Some(OptionType::StringType) },
            raw::CONFIG_TYPE_BOOL => { Some(OptionType::BooleanType) },
            _ => { None }
        }
    }
    
    /// Read value from path.
    ///
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root") {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// } 
    /// ``` 
    pub fn value<S>(&self, path : S) -> Option<OptionReader>
        where S: Into<String> {
        
        if self.element.is_none() {
            return None
        }
        
        let option = unsafe { raw::config_setting_lookup(
            self.element.unwrap(), CString::new(path.into())
                .unwrap().as_ptr())
        };
         
        if option.is_null() {
            None          
        } else {
            Some(OptionReader::new(Some(option)))
        }  
    }
    
    /// Present option value as i32.
    ///
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().as_int32() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn as_int32(&self) -> Option<i32> {
        if self.element.is_none() {
            return None
        }
        
        let result = unsafe { 
            raw::config_setting_get_int(self.element.unwrap()) 
        };
        Some(result)
    }
    
    /// Present option value as i32, return def if value not found.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// let ival = cfg.value("root.value").unwrap().as_int32_default(0);
    /// ```
    pub fn as_int32_default (&self, def : i32) -> i32 {
        match self.as_int32() {
            Some(x) => { x },
            None => { def }
        }
    }
    
    /// Present option value as i64.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().as_int64() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn as_int64(&self) -> Option<i64> {
        if self.element.is_none() {
            return None
        }
        
        let result = unsafe {
            raw::config_setting_get_int64(self.element.unwrap())
        };
        Some(result)
    }
    
    /// Present option value as i64, return def if value not exists.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// let value = cfg.value("root.value").unwrap().as_int64_default(0);
    /// ```
    pub fn as_int64_default(&self, def : i64) -> i64 {
        match self.as_int64() {
            Some(x) => { x },
            None => { def }
        }
    }
    
    /// Present option value as f64.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().as_float64() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn as_float64(&self) -> Option<f64> {
        if self.element.is_none() {
            return None
        }
        
        let result = unsafe {
            raw::config_setting_get_float(self.element.unwrap())
        };
        Some(result)
    }
    
    /// Present option value as f64, return def if value not exists.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// let value = cfg.value("root.value").unwrap().as_float64_default(0.0);
    /// ```
    pub fn as_float_default(&self, def : f64) -> f64 {
        match self.as_float64() {
            Some(x) => { x },
            None => { def }
        }
    }
    
    /// Present option value as bool.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().as_bool() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if self.element.is_none() {
            return None
        }
        
        let result = unsafe {
            raw::config_setting_get_bool(self.element.unwrap())
        };
        Some(result == raw::CONFIG_TRUE)
    }
    
    /// Present option value as bool, return def if value not exists.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// let value = cfg.value("root.value").unwrap().as_bool_default(false);
    /// ```
    pub fn as_bool_default(&self, def : bool) -> bool {
        match self.as_bool() {
            Some(x) => { x },
            None => { def }
        }
    }
    
    /// Present option value as string.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// match cfg.value("root.value").unwrap().as_string() {
    ///     Some(val) => { /* ... */ },
    ///     None => { /* ... */ }
    /// }
    /// ```
    pub fn as_string(&self) -> Option<String> {
        if self.element.is_none() {
            return None
        }
        
        let result = {
            let str = unsafe {
                raw::config_setting_get_string(self.element.unwrap())
            };

            if str.is_null() {
                return None
            } else { 
                unsafe { CStr::from_ptr(raw::config_setting_get_string(
                    self.element.unwrap())) }
            }
        };

        if result.to_str().is_ok() {
            Some(result.to_str().unwrap().to_string())
        } else {
            None
        }
    }
    
    /// Present option value as string, return def if value not exists.
    /// 
    /// # Example
    /// ```ignore
    /// use libconfig::config::Config;
    /// 
    /// let cfg = Config::new();
    /// let value = cfg.value("root.value").unwrap()
    ///     .as_string_default("default");
    /// ```
    pub fn as_string_default<S>(&self, def : S) -> String
        where S: Into<String> {
        match self.as_string() {
            Some(x) => { x },
            None => { def.into() } 
        }
    }
}