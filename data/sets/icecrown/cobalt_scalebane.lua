local card = {
    api_version = 1, id = "ICC_029", name = "Cobalt Scalebane",
    text = "At the end of your turn, give another random friendly minion +3 Attack.",
    set = "ICECROWN", type = "minion", rarity = "common",
    cost = 5, attack = 5, health = 5, tags = { "dragon" },
}

card.triggers = {{
    event = "turn_ended", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
    effect = function(ctx, self)
        local candidates = {}
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self then candidates[#candidates + 1] = minion end
        end
        if #candidates > 0 then ctx:random_entity(candidates, "cobalt_scalebane_chosen") end
    end,
}}

function card.cobalt_scalebane_chosen(ctx, self, target) cardlib.effects.buff(ctx, target, 3, 0) end

return card
