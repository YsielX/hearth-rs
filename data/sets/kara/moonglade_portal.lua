local function six_cost_minions(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 6 then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "KAR_075",
    name = "Moonglade Portal",
    text = "Restore #6 Health. Summon a random\n6-Cost minion.",
    set = "KARA",
    type = "spell",
    class = "druid",
    rarity = "rare",
    spell_school = "nature",
    cost = 6,
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    local player = ctx:controller(self)
    cardlib.effects.heal(ctx, target, 6)
    if #ctx:board(player) < 7 then
        local pool = six_cost_minions(ctx)
        if #pool > 0 then ctx:random_value(pool, "summon_six_cost_minion") end
    end
end

function card.summon_six_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
