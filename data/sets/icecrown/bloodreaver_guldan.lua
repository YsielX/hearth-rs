local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags) do
        if tag == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "ICC_831",
    name = "Bloodreaver Gul'dan",
    text = "<b>Battlecry:</b> Summon all friendly Demons that died this game.",
    set = "ICECROWN",
    type = "hero",
    class = "warlock",
    cost = 10,
    health = 30,
    armor = 5,
    hero_power = "ICC_831p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:graveyard(player)) do
        local dead = ctx:entity(entity)
        local definition = ctx:card_definition(dead.card_id)
        if definition.type == "minion" and has_tag(definition, "demon") then
            ctx:summon(player, dead.card_id)
        end
    end
end

return card
