local card = {
    api_version = 1,
    id = "RLK_505",
    name = "Marrow Manipulator",
    text = "[x]<b>Battlecry:</b> Spend up to 5\n<b>Corpses</b>. Deal 2 damage to\na random enemy for each.",
    set = "PATH_OF_ARTHAS",
    type = "minion",
    class = "death_knight",
    rarity = "rare",
    cost = 6,
    attack = 5,
    health = 5,
    tags = { "undead" },
    rune_cost = { frost = 2 },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:spend_resource_and_continue(ctx:controller(self), "corpses", 1, 5, "marrow_paid")
end

function card.marrow_paid(ctx, self, spent)
    if spent == 0 then return end
    ctx:set_data(self, "marrow_hits_left", spent)
    ctx:continue_with("marrow_choose")
end

function card.marrow_choose(ctx, self)
    if ctx:get_data(self, "marrow_hits_left") <= 0 then return end
    local enemies = ctx:enemy_characters(self)
    if #enemies > 0 then ctx:random_entity(enemies, "marrow_hit") end
end

function card.marrow_hit(ctx, self, target)
    cardlib.effects.damage(ctx, target, 2)
    local left = ctx:get_data(self, "marrow_hits_left") - 1
    ctx:set_data(self, "marrow_hits_left", left)
    if left > 0 then ctx:continue_with("marrow_choose") end
end

return card
