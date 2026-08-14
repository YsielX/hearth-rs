local function is_beast(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "beast" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_046",
    name = "King of Beasts",
    text = "[x]<b>Taunt</b>\nHas +1 Attack for each\nother Beast you control.",
    set = "GVG",
    type = "minion",
    class = "hunter",
    rarity = "rare",
    cost = 3,
    attack = 1,
    health = 5,
    tags = { "beast" },
    keywords = { "taunt" },
    auras = {
        {
            targets = function(ctx, self) return { self } end,
            attack = function(ctx, self)
                local count = 0
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    if minion ~= self and is_beast(ctx, minion) then count = count + 1 end
                end
                return count
            end,
        },
    },
}
