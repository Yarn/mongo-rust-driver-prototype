use extra::deque::Deque; 
use bson::encode::*;
use bson::json_parse::*;
use std::cmp::min;
use util::*;
//use coll::Collection;

//TODO temporary
pub struct Collection;

///Structure representing a cursor
pub struct Cursor {
    id : i64,
    //collection : @Collection,
    collection : Option<@Collection>,   // TODO temporory so tests pass
    flags : i32, // tailable, slave_ok, oplog_replay, no_timeout, await_data, exhaust, partial, can set during find() too
    skip : i32,
    limit : i32,
    open : bool,
    priv retrieved : i32,
    batch_size : i32,
    query_spec : BsonDocument,
    priv data : Deque<BsonDocument>
}

///Iterator implementation, opens access to powerful functions like collect, advance, map, etc.
impl Iterator<BsonDocument> for Cursor {
    pub fn next(&mut self) -> Option<BsonDocument> {
        if self.refresh().unwrap() == 0 || !self.open {
            return None;
        }
        Some(self.data.pop_front())
    }
}
macro_rules! query_add (
   ($obj:ident, $field:expr, $cb:ident) => {
        match $obj {
            SpecObj(doc) => {
                let mut t = BsonDocument::new();
                t.put($field, Embedded(~doc));
                self.add_query_spec(&t);
                Ok(~"added to query spec")
            }
            SpecNotation(ref s) => {
                let obj = ObjParser::from_string::<Document, ExtendedJsonParser<~[char]>>(copy *s);
                if obj.is_ok() {
                    match obj.unwrap() {
                        Embedded(ref map) => return self.$cb(SpecObj(BsonDocument::from_map(copy map.fields))),
                        _ => fail!()
                    }
                }
                else { return Err(~"could not parse json object"); }
            }
        }
   }
)
///Cursor API
impl Cursor {
    pub fn new(query: BsonDocument, collection : Option<@Collection>, id : i64, n : i32, flags : i32, vec : ~[BsonDocument]) -> Cursor {
        let mut docs = Deque::new::<BsonDocument>();
        for vec.iter().advance |&doc| {
            docs.add_back(doc);
        }

        Cursor {
            id: id,
            collection: collection,
            flags: flags,
            skip: 0,
            limit: 0,
            open: true,
            retrieved: n,
            batch_size: 0,
            query_spec: query,
            data: docs,
        }
    }
    pub fn explain(&mut self, explain: bool) {
        let mut doc = BsonDocument::new();
        doc.put(~"explain", Bool(explain));
        self.add_query_spec(&doc);
    }
    pub fn hint(&mut self, index: QuerySpec) -> Result<~str,~str> {
       query_add!(index, ~"$hint", hint) 
    }
    pub fn sort(&mut self, orderby: QuerySpec) -> Result<~str,~str> {
       query_add!(orderby, ~"$orderby", sort) 
    } 
    pub fn has_next(&self) -> bool {
        !self.data.is_empty()
    }
    pub fn close(&mut self) {
        //self.collection.db.connection.close_cursor(self.id);
        self.open = false
    }
    ///Add a flag with a bitmask
    pub fn add_flag(&mut self, mask: i32) {
        self.flags |= mask;
    }
    ///Remove a flag with a bitmask
    pub fn remove_flag(&mut self, mask: i32) {
        self.flags &= !mask;
    }
    fn add_query_spec(&mut self, doc: &BsonDocument) {
        for doc.fields.iter().advance |&(@k, @v)| {
            self.query_spec.put(k,v);
        }
    }
    fn refresh(&mut self) -> Result<i32, ~str> {
        if self.has_next() || !self.open {
            return Ok(self.data.len() as i32);
        }
        
        if self.id == 0 { //cursor is empty; query again
            let mut query_amt = self.batch_size;
            if self.limit != 0 {
                query_amt = if self.batch_size != 0 { min(self.limit, self.batch_size) } else { self.limit };
            }
            //self.send_request(asdfasdf);
            Ok(self.data.len() as i32)
        }

        else { //cursor is not empty; get_more
            let limit = if self.limit != 0 { if self.batch_size != 0 { min(self.limit - self.retrieved, self.batch_size) } else { self.limit - self.retrieved } } else { self.batch_size };
            //self.send_request(asdfasdf);
            Ok(self.data.len() as i32)
        }
    }
    /*fn send_request(&mut self, msg: Message) -> Result<~str, ~str>{
        if self.open {
            match self.collection.db.connection.send_request_and_retrieve_result(msg) {

            }
        }
        else {
            Err(~"cannot send a request through a closed cursor")
        }
    }*/
}

#[cfg(test)]
mod tests {
    extern mod bson;
    extern mod extra;

    use super::*; 
    use bson::encode::*;
    use util::*;
    //use coll::*;

    #[test]
    fn test_add_index_obj() {
        let mut doc = BsonDocument::new();
        doc.put(~"foo", Double(1f64));
        let mut cursor = Cursor::new(BsonDocument::new(), None, 0i64, 0i32, 10i32, ~[]);
        cursor.hint(SpecObj(doc));
    
        let mut spec = BsonDocument::new();    
        let mut speci = BsonDocument::new();
        speci.put(~"foo", Double(1f64));
        spec.put(~"$hint", Embedded(~speci));

        assert_eq!(cursor.query_spec, spec);
    }
    #[test]
    fn test_add_index_str() {
        let hint = ~"{\"foo\": 1}";
        let mut cursor = Cursor::new(BsonDocument::new(), None, 0i64, 0i32, 10i32, ~[]);
        cursor.hint(SpecNotation(hint));

        let mut spec = BsonDocument::new();
        let mut speci = BsonDocument::new();
        speci.put(~"foo", Double(1f64));
        spec.put(~"$hint", Embedded(~speci));
        
        assert_eq!(cursor.query_spec, spec);
    }    
}
