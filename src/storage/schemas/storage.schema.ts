import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import type { Document } from "mongoose";

export type StorageDocument = Storage & Document;

@Schema({
  timestamps: true,
})
export class Storage {
  @Prop({ required: true })
  name: string;

  @Prop()
  baseName?: string;

  @Prop()
  extName?: string;

  @Prop()
  mimeType?: string;

  @Prop()
  encoding?: string;

  @Prop()
  size?: number;

  @Prop()
  MD5Hash?: string;

  @Prop()
  iv?: string;

  @Prop({ default: "root" })
  parentId?: string;

  @Prop({ required: true })
  type: string;

  @Prop()
  userId?: string;

  @Prop()
  thumbnail?: string;

  @Prop({ default: false })
  trashed: boolean;

  @Prop()
  createdAt: Date;

  @Prop()
  updatedAt: Date;
}

export const StorageSchema = SchemaFactory.createForClass(Storage);
