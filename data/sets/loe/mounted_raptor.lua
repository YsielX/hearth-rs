local card = {
    api_version = 1, id = "LOE_050", name = "Mounted Raptor",
    text = "<b>Deathrattle:</b> Summon a random 1-Cost minion.",
    set = "LOE", type = "minion", class = "druid", rarity = "common",
    cost = 3, attack = 3, health = 2, tags = { "beast" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    if #ctx:board(ctx:controller(self)) >= 7 then return end
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 1 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_one_cost_minion") end
end

function card.summon_one_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
