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
    id = "KAR_010",
    name = "Nightbane Templar",
    text = "<b>Battlecry:</b> If you're holding a Dragon, summon two 1/1 Whelps.",
    set = "KARA",
    type = "minion",
    class = "paladin",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    if holding_dragon(ctx, self) then
        local player = ctx:controller(self)
        ctx:summon(player, "KAR_010a")
        ctx:summon(player, "KAR_010a")
    end
end

card.tokens = {{
    id = "KAR_010a",
    name = "Whelp",
    text = "",
    set = "KARA",
    type = "minion",
    class = "paladin",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "dragon" },
}}

return card
