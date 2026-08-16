local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1, id = "AT_048", name = "Healing Wave",
    text = "Restore #8 Health. Reveal a minion in each deck. If yours costs more, restore #16 instead.",
    set = "TGT", type = "spell", class = "shaman", rarity = "rare", cost = 3,
    spell_school = "nature", target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    ctx:set_data(self, "heal_target", target)
    local candidates = deck_minions(ctx, ctx:controller(self))
    if #candidates == 0 then cardlib.effects.heal(ctx, target, 8)
    else ctx:random_value(candidates, "reveal_friendly_minion") end
end

function card.reveal_friendly_minion(ctx, self, entity)
    ctx:set_data(self, "friendly_cost", ctx:entity(entity).cost)
    local candidates = deck_minions(ctx, ctx:opponent(ctx:controller(self)))
    if #candidates == 0 then cardlib.effects.heal(ctx, ctx:get_data(self, "heal_target"), 16)
    else ctx:random_value(candidates, "reveal_enemy_minion") end
end

function card.reveal_enemy_minion(ctx, self, entity)
    local amount = ctx:get_data(self, "friendly_cost") > ctx:entity(entity).cost and 16 or 8
    cardlib.effects.heal(ctx, ctx:get_data(self, "heal_target"), amount)
end

return card
