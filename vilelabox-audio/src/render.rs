use crate::deck::Deck;

/// Renderiza um trecho de reprodução offline em blocos de tamanho fixo.
///
/// Retorna os canais estéreo `(Vec<f32>, Vec<f32>)` processados deterministicamente.
pub fn render_offline(
    deck: &mut Deck,
    total_frames: usize,
    block_size: usize,
) -> (Vec<f32>, Vec<f32>) {
    let mut out_l = vec![0.0f32; total_frames];
    let mut out_r = vec![0.0f32; total_frames];

    let block_sz = block_size.max(1);
    let mut offset = 0;

    let mut buf_l = vec![0.0f32; block_sz];
    let mut buf_r = vec![0.0f32; block_sz];

    while offset < total_frames {
        let chunk = (total_frames - offset).min(block_sz);
        deck.process_block(&mut buf_l[..chunk], &mut buf_r[..chunk]);
        out_l[offset..offset + chunk].copy_from_slice(&buf_l[..chunk]);
        out_r[offset..offset + chunk].copy_from_slice(&buf_r[..chunk]);
        offset += chunk;
    }

    (out_l, out_r)
}
