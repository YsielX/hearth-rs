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
    rarity = "legendary",
    cost = 10,
    health = 30,
    armor = 5,
    hero_power = "ICC_831p",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    for _, card_id in ipairs(ctx:minions_died(player)) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and has_tag(definition, "demon") then
            ctx:summon(player, card_id)
        end
    end
end

return card
