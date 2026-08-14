local card = {
    api_version = 1, id = "LOE_021", name = "Dart Trap",
    text = "<b>Secret:</b> After an opposing Hero Power is used, deal $5 damage to a random enemy.",
    set = "LOE", type = "spell", class = "hunter", rarity = "common",
    cost = 2, keywords = { "secret" },
}

local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

card.triggers = {{
    event = "hero_power_used", timing = "after", active_zones = { "secret" },
    condition = function(ctx, self, event)
        return event.player ~= ctx:controller(self)
    end,
    effect = function(ctx, self)
        ctx:reveal_secret(self)
        local candidates = {}
        for _, enemy in ipairs(ctx:enemy_characters(self)) do
            if not dormant(ctx, enemy) then candidates[#candidates + 1] = enemy end
        end
        if #candidates > 0 then ctx:random_entity(candidates, "deal_dart_damage") end
    end,
}}

function card.deal_dart_damage(ctx, self, target) ctx:damage(target, 5) end
return card
