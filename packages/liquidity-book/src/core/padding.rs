use cosmwasm_std::{Binary, Response, StdResult};

/// Take a Vec<u8> and pad it up to a multiple of `block_size`, using spaces at the end.
pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}

/// Pad the data and logs in a `Response` to the block size, with spaces.
// The big `where` clause is based on the `where` clause of `Response`.
// Users don't need to care about it as the type `T` has a default, and will
// always be known in the context of the caller.
pub fn pad_handle_result(response: StdResult<Response>, block_size: usize) -> StdResult<Response> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(&mut data.0, block_size);
            data
        });
        for log in &mut response.attributes {
            // Safety: These two are safe because we know the characters that
            // `space_pad` appends are valid UTF-8
            unsafe { space_pad(log.key.as_mut_vec(), block_size) };
            unsafe { space_pad(log.value.as_mut_vec(), block_size) };
        }
        response
    })
}

/// Pad a `StdResult<Binary>` with spaces
pub fn pad_query_result(response: StdResult<Binary>, block_size: usize) -> StdResult<Binary> {
    response.map(|mut response| {
        space_pad(&mut response.0, block_size);
        response
    })
}
