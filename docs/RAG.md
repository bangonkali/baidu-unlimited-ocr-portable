# RAG Pipeline Notes

Trapo RAG execution is currently a three-step pipeline:

1. Start Ingest
2. Text Index
3. Generate Embedding

DuckDB initialization best-effort loads `fts` and `vss` for text and vector search. DuckPGQ capability is recorded but graph query execution is deferred until a graph-backed retrieval feature is introduced.

The task table is designed as a queue (`pipeline_tasks`) with runner identity and task status, but this release intentionally permits only one global active task at a time across ingest, text indexing, and embedding generation. Future runner agents can claim queued tasks in parallel once scheduling, isolation, and UI conflict handling are implemented.

Embedding generation uses local GGUF files through llama.cpp. Each supported embedding model stores its tuned pooling, context, batch, GPU-layer, normalization, prefix, and dimension metadata in `rag_embedding_models`; each embedding run records the model and dimension used so the search UI can query only against compatible generated vectors.

Text indexing materializes page OCR into bounded `page_chunk` RAG segments before FTS or embedding work. Chunks use conservative token estimates, including CJK-aware counting for no-whitespace OCR output. Embedding execution also checks actual llama.cpp token counts and splits oversized document segments into sub-embeddings before native decode, so tokenizer-specific surprises do not fail the whole embedding run.
