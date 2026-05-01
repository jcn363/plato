//! Vector embedding engine for semantic search using candle-core

use crate::traits::VectorEmbedder;
use crate::PlatoResult;
use candle_core::{Device, Module, Tensor};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

/// A local embedding engine powered by candle-core
pub struct CandleEmbedder {
    device: Device,
    tokenizer: Tokenizer,
    model_weights: candle_nn::Embedding,
}

impl CandleEmbedder {
    /// Initialize a new embedder, loading the tokenizer and model weights
    pub fn new(model_path: &str, tokenizer_path: &str) -> Plati...
        let device = Device::Cpu;
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;

        // Load weights from safetensors
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_path], candle_core::DType::F32, &device)?
        };
        // Assuming a standard transformer embedding layer name
        let model_weights = candle_nn::embedding(30522, 384, vb.pp("embeddings.word_embeddings"))?;

        Ok(Self {
            device,
            tokenizer,
            model_weights,
        })
    }
}

impl VectorEmbedder for CandleEmbedder {
    fn embed(&self, text: &str) -> crate::AiResult<Vec<f32>> {
        // Tokenize text
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;
        let ids = encoding.get_ids();

        // Inference: Create input tensor and run through model
        let input = Tensor::new(ids, &self.device)?.unsqueeze(0)?;
        let output = self.model_weights.forward(&input)?;

        // Mean pooling: average embeddings over sequence length
        let embedding = output.mean(1)?.flatten_all()?.to_vec1::<f32>()?;

        Ok(embedding)
    }
}
