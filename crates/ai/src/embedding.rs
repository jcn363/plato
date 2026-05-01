//! Vector embedding engine for semantic search using candle-core

use std::sync::Arc;
use anyhow::{Result};
use candle_core::{Device, Tensor, Module};
use candle_nn::{VarBuilder, Embedding};
use tokenizers::Tokenizer;
use crate::traits::VectorEmbedder;

/// A local embedding engine powered by candle-core
#[derive(Clone)]
pub struct CandleEmbedder {
    device: Device,
    tokenizer: Tokenizer,
    model_weights: Arc<Embedding>, 
}

impl CandleEmbedder {
    /// Initialize a new embedder, loading the tokenizer and model weights
    pub fn new(model_path: &str, tokenizer_path: &str) -> Result<Self> {
        let device = Device::Cpu;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;
        
        // SAFETY: VarBuilder::from_mmaped_safetensors uses mmap to map the model file into memory.
        // It assumes the model file is not modified externally while being read, which is standard for 
        // read-only model weight files.
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], candle_core::DType::F32, &device)?
        };
        let model_weights = candle_nn::embedding(30522, 384, vb.pp("embeddings.word_embeddings"))?;
        
        Ok(Self { device, tokenizer, model_weights: Arc::new(model_weights) })
    }

    /// Return a reference to the model weights
    pub fn model(&self) -> &Embedding {
        &self.model_weights
    }
}

impl VectorEmbedder for CandleEmbedder {
    fn embed(&self, text: &str) -> plato_error::PlatoResult<Vec<f32>> {
        let encoding = self.tokenizer.encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;
        let ids = encoding.get_ids();
        
        let input = Tensor::new(ids, &self.device)?.unsqueeze(0)?;
        let output = self.model_weights.forward(&input)?;
        
        let embedding = output.mean(1)?.flatten_all()?.to_vec1::<f32>()?;
        
        Ok(embedding)
    }
}
