local function weapon_pool(ctx, current)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(id).type == "weapon" and id ~= current then pool[#pool + 1] = id end
    end
    return pool
end
local card = {
    api_version = 1, id = "UNG_929", name = "Molten Blade",
    text = "Transforms into a new weapon when in hand that changes each turn.",
    set = "UNGORO", type = "weapon", class = "warrior", rarity = "rare", cost = 1, attack = 1, health = 1,
    triggers = {{
        event = "turn_started", timing = "after", active_zones = { "hand" },
        condition = function(ctx, self, event) return event.player == ctx:controller(self) end,
        effect = function(ctx, self)
            local pool = weapon_pool(ctx, ctx:entity(self).card_id)
            if #pool > 0 then ctx:attach_script(self, "UNG_929"); ctx:random_value(pool, "become_weapon") end
        end,
    }},
}
function card.become_weapon(ctx, self, id) ctx:transform_preserving_scripts(self, id) end
return card
