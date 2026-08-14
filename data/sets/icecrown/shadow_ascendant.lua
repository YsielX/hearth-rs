local card = {
    api_version = 1, id = "ICC_210", name = "Shadow Ascendant",
    text = "[x]At the end of your turn,\ngive another random\nfriendly minion +1/+1.",
    set = "ICECROWN", type = "minion", class = "priest", rarity = "common",
    cost = 2, attack = 2, health = 3, tags = { "undead" },
}

card.triggers = {{
    event = "turn_ended", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and #ctx:friendly_minions(self) > 1
    end,
    effect = function(ctx, self)
        local pool = {}
        for _, entity in ipairs(ctx:friendly_minions(self)) do
            if entity ~= self then pool[#pool + 1] = entity end
        end
        if #pool > 0 then ctx:random_entity(pool, "ascend_minion") end
    end,
}}

function card.ascend_minion(ctx, self, target) ctx:buff(target, 1, 1) end

return card
