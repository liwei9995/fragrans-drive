import {
  Injectable,
  Logger,
  type OnApplicationShutdown,
  type OnModuleInit,
} from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import type { DeleteResult } from "mongodb";
import type { Model } from "mongoose";
import type { CreateUserDto } from "./dto/create-user.dto";
import { User, type UserDocument } from "./schemas/user.schema";

@Injectable()
export class UsersService implements OnModuleInit, OnApplicationShutdown {
  private readonly logger = new Logger(UsersService.name);

  constructor(@InjectModel(User.name) private userModel: Model<UserDocument>) {}

  onModuleInit() {
    this.logger.log("UsersService dependencies initialized");
  }

  onApplicationShutdown(signal: string) {
    this.logger.log(`Received application shutdown signal: ${signal}`); // e.g. "SIGNINT"
  }

  async findAll(select = "-password"): Promise<User[]> {
    return this.userModel.find().select(select).exec();
  }

  async findOne(query = {}, select = "-password"): Promise<User | undefined> {
    return this.userModel.findOne(query).select(select).lean();
  }

  async findOneAndUpdate(query = {}, update = {}, options = {}): Promise<User> {
    return this.userModel.findOneAndUpdate(query, update, {
      ...options,
      new: true,
      fields: "-password",
    });
  }

  async create(createUserDto: CreateUserDto): Promise<any> {
    const createdUser = new this.userModel(createUserDto);

    return (await createdUser.save()).toJSON();
  }

  async deleteOne(id: string): Promise<User> {
    return this.userModel.findOneAndRemove({ _id: id });
  }

  async deleteAll(): Promise<DeleteResult> {
    return this.userModel.deleteMany();
  }

  async updatePassword(id: string, password: string): Promise<string> {
    const user = await this.userModel.findById(id);

    if (!user) {
      return "User not found.";
    }

    user.password = password;
    user.save();

    return "Password has been changed.";
  }
}
