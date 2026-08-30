local card = {
    api_version = 1,
    id = "KAR_077",
    name = "Silvermoon Portal",
    text = "Give a minion +2/+2. Summon a random\n2-Cost minion.",
    set = "KARA",
    type = "spell",
    class = "paladin",
    rarity = "common",
    spell_school = "holy",
    cost = 3,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.buff(ctx, target, 2, 2)
    if #ctx:board(ctx:controller(self)) >= 7 then return end

    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.cost == 2 then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_minion") end
end

function card.summon_minion(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
