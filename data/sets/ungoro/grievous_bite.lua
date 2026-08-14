return { api_version = 1, id = "UNG_910", name = "Grievous Bite",
    text = "Deal $3 damage to a minion and $1 damage to adjacent ones.", set = "UNGORO",
    type = "spell", class = "hunter", rarity = "common", cost = 2,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target)
        local hits = { { target, 3 } }
        for _, adjacent in ipairs(ctx:adjacent_minions(target)) do hits[#hits + 1] = { adjacent, 1 } end
        ctx:damage_batch(hits)
    end }
