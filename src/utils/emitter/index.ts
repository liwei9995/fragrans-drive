import mitt, { Emitter } from 'mitt'

type Events = Record<string, any>

const emitter: Emitter<Events> = mitt<Events>()

export default emitter
