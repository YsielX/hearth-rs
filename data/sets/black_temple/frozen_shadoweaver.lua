local function enemies(ctx, self)
    local result = {}
    local controller = ctx:controller(self)
    for _, character in ipairs(ctx:characters()) do
        if ctx:controller(character) ~= controller then
            result[#result + 1] = character
        end
    end
    return result
end

return {
    api_version = 1,
    id = "BT_714",
    name = "Frozen Shadoweaver",
    text = "<b>Battlecry:</b> <b>Freeze</b> an enemy.",
    set = "BLACK_TEMPLE",
    type = "minion",
    cost = 3,
    attack = 4,
    health = 3,
    tags = { "draenei" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = enemies,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:freeze(target) end
    end,
}
