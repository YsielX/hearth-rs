local card = {
    api_version = 1,
    id = "KAR_070",
    name = "Ethereal Peddler",
    text = "<b>Battlecry:</b> If you're holding any cards from another class, reduce their Cost by (2).",
    set = "KARA",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 6,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local own_class = ctx:player(player).class
    for _, entity in ipairs(ctx:hand(player)) do
        local definition = ctx:card_definition(ctx:entity(entity).card_id)
        if definition.class ~= "neutral" and definition.class ~= own_class then
            ctx:modify(entity, { stat = "cost", operation = "add", value = -2 })
        end
    end
end

return card
