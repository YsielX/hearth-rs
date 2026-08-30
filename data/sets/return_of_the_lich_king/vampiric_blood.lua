local card = {
    api_version = 1,
    id = "RLK_051",
    name = "Vampiric Blood",
    text = "[x]Give your hero +5 Health.\nSpend 3 <b>Corpses</b> to gain\n5 more and draw a card.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    cost = 2,
    rune_cost = { blood = 3 },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local hero = ctx:player(player).hero
    cardlib.effects.buff(ctx, hero, 0, 5)
    ctx:spend_resource_and_continue(player, "corpses", 3, 3, "vampiric_bonus")
end

function card.vampiric_bonus(ctx, self, spent)
    if spent == 0 then return end
    local player = ctx:controller(self)
    cardlib.effects.buff(ctx, ctx:player(player).hero, 0, 5)
    ctx:draw(player, 1)
end

return card
