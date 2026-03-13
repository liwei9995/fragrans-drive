import {
  createCipheriv,
  createDecipheriv,
  randomBytes,
  scrypt,
} from "node:crypto";
import { promisify } from "node:util";

export const getIV = (len = 16) => randomBytes(len).toString("hex");

export const getKey = async (password: string, len = 32) =>
  await promisify(scrypt)(password, "salt", len);

export const encryptBuffer = (
  key: string,
  iv: string,
  contentToEncrypt: Buffer | string,
  algorithm = "aes-256-ctr",
) => {
  const cipher = createCipheriv(algorithm, key, Buffer.from(iv, "hex"));

  return Buffer.concat([cipher.update(contentToEncrypt), cipher.final()]);
};

export const decryptStream = (
  key: string,
  iv: string,
  stream,
  algorithm = "aes-256-ctr",
) => {
  const decipher = createDecipheriv(algorithm, key, Buffer.from(iv, "hex"));

  return stream?.pipe(decipher);
};
