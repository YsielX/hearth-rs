local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords or {}) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "ICC_245", name = "Blackguard",
    text = "Whenever your hero is healed, deal that much damage to a random enemy minion.",
    set = "ICECROWN", type = "minion", class = "paladin", rarity = "epic",
    cost = 6, attack = 3, health = 9, tags = { "undead", "draenei" },
}

card.triggers = {{
    event = "healed", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.amount > 0 and event.target == ctx:player(ctx:controller(self)).hero
    end,
    effect = function(ctx, self, event)
        local choices = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" and not dormant(ctx, entity) then
                choices[#choices + 1] = { target = entity, amount = event.amount }
            end
        end
        if #choices > 0 then ctx:random_value(choices, "blackguard_hit") end
    end,
}}

function card.blackguard_hit(ctx, self, choice)
    cardlib.effects.damage(ctx, choice.target, choice.amount)
end

return card
