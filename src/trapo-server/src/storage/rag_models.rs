impl Repository {
    pub(crate) async fn upsert_rag_embedding_models(
        &self,
        models: &[RagEmbeddingModelRow],
    ) -> Result<()> {
        let models = models.to_vec();
        self.with_write(move |mut conn| {
            let transaction = conn.transaction()?;
            for model in &models {
                transaction.execute(
                    "INSERT INTO rag_embedding_models(
                       model_id, display_name, provider, repo_id, filename, revision,
                       routing_origin, model_family, dimension, context_tokens, pooling, normalize,
                       query_prefix, document_prefix, llama_params_json, recommended_vram_gb, active, updated_at
                     )
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CAST(now() AS VARCHAR))
                     ON CONFLICT(model_id) DO UPDATE SET
                       display_name = excluded.display_name,
                       provider = excluded.provider,
                       repo_id = excluded.repo_id,
                       filename = excluded.filename,
                       revision = excluded.revision,
                       routing_origin = excluded.routing_origin,
                       model_family = excluded.model_family,
                       dimension = excluded.dimension,
                       context_tokens = excluded.context_tokens,
                       pooling = excluded.pooling,
                       normalize = excluded.normalize,
                       query_prefix = excluded.query_prefix,
                       document_prefix = excluded.document_prefix,
                       llama_params_json = excluded.llama_params_json,
                       recommended_vram_gb = excluded.recommended_vram_gb,
                       active = excluded.active,
                       updated_at = CAST(now() AS VARCHAR)",
                    params![
                        model.model_id.as_str(),
                        model.display_name.as_str(),
                        model.provider.as_str(),
                        model.repo_id.as_str(),
                        model.filename.as_str(),
                        model.revision.as_str(),
                        model.routing_origin.as_str(),
                        model.model_family.as_str(),
                        i64::from(model.dimension),
                        i64::from(model.context_tokens),
                        model.pooling.as_str(),
                        model.normalize,
                        model.query_prefix.as_str(),
                        model.document_prefix.as_str(),
                        model.llama_params.to_string().as_str(),
                        model.recommended_vram_gb,
                        model.active
                    ],
                )?;
            }
            transaction.commit()?;
            Ok(())
        })
        .await
    }

    pub(crate) async fn list_rag_embedding_models(&self) -> Result<Vec<RagEmbeddingModelRow>> {
        self.with_read(|conn| {
            let mut statement = conn.prepare(
                "SELECT model_id, display_name, provider, repo_id, filename, revision,
                  routing_origin, model_family, dimension, context_tokens, pooling, normalize,
                  query_prefix, document_prefix, llama_params_json, recommended_vram_gb, active
                 FROM rag_embedding_models
                 WHERE active = true
                 ORDER BY recommended_vram_gb, display_name",
            )?;
            let rows = statement.query_map([], rag_embedding_model_from_row)?;
            collect_rows(rows)
        })
        .await
    }

    pub(crate) async fn list_used_rag_embedding_models(
        &self,
    ) -> Result<Vec<RagEmbeddingModelRow>> {
        self.with_read(|conn| {
            let mut statement = conn.prepare(
                "SELECT DISTINCT m.model_id, m.display_name, m.provider, m.repo_id, m.filename,
                  m.revision, m.routing_origin, m.model_family, m.dimension, m.context_tokens,
                  m.pooling, m.normalize, m.query_prefix, m.document_prefix,
                  m.llama_params_json, m.recommended_vram_gb, m.active
                 FROM rag_embedding_models m
                 JOIN rag_embedding_runs r ON r.model_id = m.model_id
                 WHERE r.status = 'completed'
                 ORDER BY m.display_name",
            )?;
            let rows = statement.query_map([], rag_embedding_model_from_row)?;
            collect_rows(rows)
        })
        .await
    }
}
