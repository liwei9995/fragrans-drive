import * as crypto from "node:crypto";
import { isFunction, isString, isUndefined } from "./index";
import isStream from "./isStream";

const md5 = (obj) => {
  if (obj === null || isUndefined(obj)) {
    return Promise.resolve(null);
  }

  if (!isStream(obj) && !isString(obj) && !Buffer.isBuffer(obj)) {
    if (!isFunction(obj.toString)) {
      return Promise.resolve(null);
    }
    obj = obj.toString();
  }

  return new Promise((res, rej) => {
    const hash = crypto.createHash("md5");

    if (isStream(obj)) {
      obj.on("data", (d) => {
        hash.update(d);
      });

      obj.on("end", () => {
        res(hash.digest("hex"));
      });

      obj.on("error", (err) => {
        rej(err);
      });

      return;
    }

    hash.update(obj, "utf8");
    res(hash.digest("hex"));
  });
};

export default md5;
