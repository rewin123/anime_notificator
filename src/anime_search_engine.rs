use super::*;
use serde_json;
use tantivy::IndexReader;
use tantivy::LeasedItem;
use tantivy::Searcher;
use std::fs;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tempfile::TempDir;

pub struct AnimeSearchEngine {
    schema : Schema,
    title : Field,
    desc : Field,
    url : Field,
    index : Index,
    index_path : TempDir,
    reader : IndexReader,
    searcher : LeasedItem<Searcher>,
    query_parser : QueryParser
}

impl Default for AnimeSearchEngine {
    fn default() -> Self {
        let index_path = TempDir::new().unwrap();
    let mut scheme_builder = Schema::builder();
    scheme_builder.add_text_field("title", TEXT | STORED);
    scheme_builder.add_text_field("desc", TEXT | STORED);
    scheme_builder.add_text_field("url", TEXT | STORED);

    let schema = scheme_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone()).unwrap();
    let mut index_writer = index.writer(50000000).unwrap();

    let title = schema.get_field("title").unwrap();
    let desc = schema.get_field("desc").unwrap();
    let url = schema.get_field("url").unwrap();

    let anime_read = fs::read("loaded_ongoings.json").unwrap();
    
    let animes = serde_json::from_str::<Vec<BigAnime>>(&String::from_utf8_lossy(&anime_read)).unwrap();

    for anime in animes {
        // println!("Anime: {}", anime.name);
        let mut doc = Document::default();
        doc.add_text(title, anime.name);
        doc.add_text(desc, anime.desc);
        doc.add_text(url, anime.url);

        index_writer.add_document(doc).unwrap();
    }

    index_writer.commit().unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![title, desc]);

    Self { schema, title, desc, url, index, index_path, reader, searcher, query_parser }

    }
}

impl AnimeSearchEngine {
    pub fn find_top(&self, request : &String, count : usize) -> Vec<BigAnime> {
        let mut res = vec![];

        let query = self.query_parser.parse_query(request).unwrap();
        let top_docs = self.searcher.search(&query, &TopDocs::with_limit(count)).unwrap();
        for (score, doc_address) in top_docs {
            let rel_doc = self.searcher.doc(doc_address).unwrap();
            let anime_name = rel_doc.get_first(self.title).unwrap().as_text().unwrap().to_string();
            let anime_desc = rel_doc.get_first(self.desc).unwrap().as_text().unwrap().to_string();
            let anime_url = rel_doc.get_first(self.url).unwrap().as_text().unwrap().to_string();

            res.push(
                BigAnime {
                    name : anime_name,
                    desc : anime_desc,
                    url : anime_url,
                    episode : 0
                }
            );
        }

        res
    }
}