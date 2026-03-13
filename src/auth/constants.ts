export const jwtConstants = {
  secret: process.env.JWT_SECRET || "dev-only-change-in-production",
  expiresIn: process.env.JWT_EXPIRES_IN || "6000s",
};
