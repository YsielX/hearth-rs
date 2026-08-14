return { api_version = 1, id = "UNG_955", name = "Meteor",
    text = "Deal $15 damage to a minion and $4 damage to adjacent ones.", set = "UNGORO",
    type = "spell", class = "mage", rarity = "epic", spell_school = "fire", cost = 6,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local hits = { { target, 15 } }
        for _, adjacent in ipairs(ctx:adjacent_minions(target)) do hits[#hits + 1] = { adjacent, 4 } end
        ctx:damage_batch(hits)
    end }
