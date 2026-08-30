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
    ctx:buff(hero, 0, 5)
    if ctx:spend_corpses(player, 3) then
        ctx:buff(hero, 0, 5)
        ctx:draw(player, 1)
    end
end

return card
