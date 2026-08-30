local card = {
    api_version = 1,
    id = "RLK_083",
    name = "Deathchiller",
    text = "[x]After you cast a spell,\ndeal 1 damage to two\nrandom enemies.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "elemental" },
    rune_cost = { frost = 2 },
}

local function choose_enemy(ctx, self)
    local enemies = ctx:enemy_characters(self)
    if #enemies > 0 then ctx:random_entity(enemies, "deathchiller_hit") end
end

card.triggers = {{
    event = "spell_cast",
    timing = "after",
    active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast
    end,
    effect = function(ctx, self)
        ctx:set_data(self, "deathchiller_hits_left", 2)
        ctx:continue_with("deathchiller_choose")
    end,
}}

function card.deathchiller_choose(ctx, self)
    if ctx:get_data(self, "deathchiller_hits_left") > 0 then choose_enemy(ctx, self) end
end

function card.deathchiller_hit(ctx, self, target)
    cardlib.effects.damage(ctx, target, 1)
    local left = ctx:get_data(self, "deathchiller_hits_left") - 1
    ctx:set_data(self, "deathchiller_hits_left", left)
    if left > 0 then ctx:continue_with("deathchiller_choose") end
end

return card
