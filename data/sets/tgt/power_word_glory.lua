local function has_glory(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "power_word_glory" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "AT_013",
    name = "Power Word: Glory",
    text = "Choose a minion. Whenever it attacks, restore #4 Health to\nyour hero.",
    set = "TGT",
    type = "spell",
    class = "priest",
    rarity = "common",
    spell_school = "holy",
    cost = 1,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    triggers = {
        {
            event = "attack",
            timing = "before",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                local target = ctx:get_data(self, "glory_target")
                return target ~= 0 and event.attacker == target and has_glory(ctx, target)
            end,
            effect = function(ctx, self)
                local player = ctx:controller(self)
                cardlib.effects.heal(ctx, ctx:player(player).hero, 4)
            end,
        },
    },
}

function card.on_play(ctx, self, target)
    ctx:set_data(self, "glory_target", target)
    ctx:grant_keyword(target, "power_word_glory")
end

return card
