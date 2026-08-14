local card = {
    api_version = 1,
    id = "KAR_076",
    name = "Firelands Portal",
    text = "Deal $6 damage. Summon a random\n6-Cost minion.",
    set = "KARA",
    type = "spell",
    class = "mage",
    spell_school = "fire",
    rarity = "common",
    cost = 7,
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    ctx:damage(target, 6)
    ctx:continue_with("choose_six_cost_minion")
end

function card.choose_six_cost_minion(ctx, self)
    if #ctx:board(ctx:controller(self)) >= 7 then return end
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 6 then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "summon_six_cost_minion") end
end

function card.summon_six_cost_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
