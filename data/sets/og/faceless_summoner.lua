local card = {
    api_version = 1, id = "OG_207", name = "Faceless Summoner",
    text = "<b>Battlecry:</b> Summon a random 3-Cost minion.", set = "OG",
    type = "minion", class = "mage", rarity = "common", cost = 6,
    attack = 5, health = 5, keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    if #ctx:board(ctx:controller(self)) >= 7 then return end
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 3 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_three_cost_minion") end
end
function card.summon_three_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end
return card
