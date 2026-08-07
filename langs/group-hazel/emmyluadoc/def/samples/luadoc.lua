---@class Person
---@field name string
---@field age number
local Person = {}

---Creates a new person
---@param name string The person's name
---@param age number The person's age
---@return Person The created person instance
function Person.new(name, age)
    local self = setmetatable({}, { __index = Person })
    self.name = name
    self.age = age
    return self
end

---@param person Person
---@return string
function Person.greet(person)
    return "Hello, " .. person.name .. "!"
end
