let newStruct = _ => class {}
let create = Struct => new Struct
let is = (instance, Struct) => instance instanceof Struct

let Point = newStruct()
let p = create(Point)
