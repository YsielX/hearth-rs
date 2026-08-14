local card = {
    api_version = 1,
    id = "KAR_009",
    name = "Babbling Book",
    text = "<b>Battlecry:</b> Add a random Mage spell to your hand.",
    set = "KARA",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 1,
    attack = 1,
    health = 2,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell" and definition.class == "mage" then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "receive_mage_spell") end
end

function card.receive_mage_spell(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
