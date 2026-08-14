local function four_cost_minions(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 4 then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "KAR_091",
    name = "Ironforge Portal",
    text = "Gain 4 Armor.\nSummon a random\n4-Cost minion.",
    set = "KARA",
    type = "spell",
    class = "warrior",
    rarity = "common",
    spell_school = "fire",
    cost = 4,
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    ctx:gain_armor(player, 4)
    if #ctx:board(player) < 7 then
        local pool = four_cost_minions(ctx)
        if #pool > 0 then ctx:random_value(pool, "summon_four_cost_minion") end
    end
end

function card.summon_four_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
