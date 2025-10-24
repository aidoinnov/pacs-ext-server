//! # Basic PACS Server Library
//! 
//! Clean Architecture 패턴을 따르는 기초 백엔드 서버 라이브러리입니다.

// 모듈 선언
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// 공개 API
pub use actix_web::{web, App, HttpResponse, HttpServer, Responder};
