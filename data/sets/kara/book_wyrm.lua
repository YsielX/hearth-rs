local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == wanted or tag == "all" then return true end
    end
    return false
end

local function holding_dragon(ctx, self)
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if entity ~= self and has_tag(ctx:card_definition(ctx:entity(entity).card_id), "dragon") then
            return true
        end
    end
    return false
end

local card = {
    api_version = 1,
    id = "KAR_033",
    name = "Book Wyrm",
    text = "<b>Battlecry:</b> If you're holding a Dragon, destroy an enemy minion with 3 or less Attack.",
    set = "KARA",
    type = "minion",
    rarity = "rare",
    cost = 6,
    attack = 3,
    health = 6,
    tags = { "dragon" },
    keywords = { "battlecry" },
    target_mode = "required_if_available",
}

function card.targets(ctx, self)
    if not holding_dragon(ctx, self) then return {} end
    local result = {}
    for _, minion in ipairs(ctx:enemy_minions(self)) do
        if ctx:entity(minion).attack <= 3 then result[#result + 1] = minion end
    end
    return result
end

function card.on_battlecry(ctx, self, target)
    if target then cardlib.effects.destroy(ctx, target) end
end

return card
