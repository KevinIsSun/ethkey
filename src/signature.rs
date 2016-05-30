use std::ops::{Deref, DerefMut};
use std::mem;
use secp256k1::{Message, RecoverableSignature, RecoveryId, Error as SecpError};
use secp256k1::key::{SecretKey, PublicKey};
use {Secret, Public, SECP256K1, Error};

#[repr(C)]
pub struct Signature {
	pub r: [u8; 32],
	pub s: [u8; 32],
	pub v: u8,
}

impl Default for Signature {
	fn default() -> Self {
		Signature {
			r: [0u8; 32],
			s: [0u8; 32],
			v: 0u8,
		}
	}
}

impl From<[u8; 65]> for Signature {
	fn from(s: [u8; 65]) -> Self {
		unsafe { mem::transmute(s) }
	}
}

impl Into<[u8; 65]> for Signature {
	fn into(self) -> [u8; 65] {
		unsafe { mem::transmute(self) }
	}
}

impl Deref for Signature {
	type Target = [u8; 65];

	fn deref(&self) -> &Self::Target {
		unsafe { mem::transmute(self) }
	}
}

impl DerefMut for Signature {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { mem::transmute(self) }
	}
}

pub fn sign(secret: &Secret, message: &[u8; 32]) -> Result<Signature, Error> {
	let context = &SECP256K1;
	// no way to create from raw byte array.
	let sec: &SecretKey = unsafe { mem::transmute(secret) };
	let s = try!(context.sign_recoverable(&try!(Message::from_slice(message)), sec));
	let (rec_id, data) = s.serialize_compact(context);
	let mut signature = Signature::default();
	signature.r.copy_from_slice(&data[0..32]);
	// no need to check if s is low, it alawys is
	signature.s.copy_from_slice(&data[32..64]);
	signature.v = rec_id.to_i32() as u8;
	Ok(signature)
}

pub fn verify(public: &Public, signature: &Signature, message: &[u8; 32]) -> Result<bool, Error> {
	let context = &SECP256K1;
	let rsig = try!(RecoverableSignature::from_compact(context, &signature[0..64], try!(RecoveryId::from_i32(signature[64] as i32))));
	let sig = rsig.to_standard(context);

	let pdata: [u8; 65] = {
		let mut temp = [4u8; 65];
		(&mut temp[1..65]).copy_from_slice(public);
		temp
	};

	let publ = try!(PublicKey::from_slice(context, &pdata));
	match context.verify(&try!(Message::from_slice(message)), &sig, &publ) {
		Ok(_) => Ok(true),
		Err(SecpError::IncorrectSignature) => Ok(false),
		Err(x) => Err(Error::from(x))
	}
}

#[cfg(test)]
mod tests {
	use {Generator, Random};
	use super::{sign, verify};

	#[test]
	fn sign_and_verify() {
		let keypair = Random.generate().unwrap();
		let message = [1u8; 32];
		let signature = sign(keypair.secret(), &message).unwrap();
		assert!(verify(keypair.public(), &signature, &message).unwrap());
	}
}
