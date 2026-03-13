import * as stream from "node:stream";

const isStream = (obj) => {
  return obj instanceof stream.Stream;
};

export default isStream;
