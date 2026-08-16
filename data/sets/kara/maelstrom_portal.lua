local card = {
    api_version = 1,
    id = "KAR_073",
    name = "Maelstrom Portal",
    text = "Deal $1 damage to all enemy minions. Summon a random\n1-Cost minion.",
    set = "KARA",
    type = "spell",
    class = "shaman",
    spell_school = "nature",
    rarity = "rare",
    cost = 2,
}

function card.on_play(ctx, self)
    local enemies = ctx:enemy_minions(self)
    if #enemies > 0 then cardlib.effects.damage_all(ctx, enemies, 1) end
    ctx:continue_with("choose_one_cost_minion")
end

function card.choose_one_cost_minion(ctx, self)
    if #ctx:board(ctx:controller(self)) >= 7 then return end
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 1 then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "summon_one_cost_minion") end
end

function card.summon_one_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
