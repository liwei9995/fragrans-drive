import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import * as bcrypt from "bcryptjs";
import type { Document } from "mongoose";
import { Role } from "../../common/enums/role.enum";

export type UserDocument = User & Document;

@Schema({
  timestamps: true,
})
export class User {
  @Prop({ required: true, unique: true })
  email: string;

  @Prop({ required: true })
  password: string;

  @Prop()
  firstName: string;

  @Prop()
  lastName: string;

  @Prop()
  gender?: number;

  @Prop()
  age?: number;

  @Prop()
  avatar?: string;

  @Prop({
    type: Array,
    enum: Role,
    default: [Role.User],
  })
  roles?: Role[];
}

export const UserSchema = SchemaFactory.createForClass(User);

type comparePasswordFunction = (
  candidatePassword: string,
  cb: (err: any, isMatch: any) => void,
) => void;

const comparePassword: comparePasswordFunction = function (
  candidatePassword,
  cb,
) {
  bcrypt.compare(
    candidatePassword,
    this.password,
    (err: Error, isMatch: boolean) => {
      cb(err, isMatch);
    },
  );
};

UserSchema.pre("save", async function save(next) {
  const user = this as UserDocument;
  const saltOrRounds = 10;

  if (!user.isModified("password")) {
    return next();
  }

  try {
    const hash = await bcrypt.hash(user.password, saltOrRounds);

    user.password = hash;
  } catch (err) {
    return next(err);
  }
});

UserSchema.methods.comparePassword = comparePassword;
